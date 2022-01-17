use nom::bytes::complete::take;
use nom::error::ErrorKind;
use nom::number::complete::{be_u16, be_u32, be_u8};
use nom::{Err, IResult, Slice};

use crate::attribute_info::types::StackMapFrame::*;
use crate::attribute_info::*;

use crate::constant_info::ConstantInfo;
use crate::parser::ParseData;
use crate::util::{constant_pool_index_raw, count_sv, skip_count};

pub fn skip_attribute_parser(i: ParseData) -> IResult<ParseData, ()> {
    let (i, _) = constant_pool_index_raw::<ConstantInfo>(i)?;
    let (i, attribute_length) = be_u32(i)?;
    let (i, _) = take(attribute_length)(i)?;
    Ok((i, ()))
}

pub fn attribute_parser(i: ParseData) -> IResult<ParseData, AttributeInfo> {
    let (i, attribute_name_index) = constant_pool_index_raw(i)?;
    let (i, attribute_length) = be_u32(i)?;
    let (i, info) = take(attribute_length)(i)?;
    Ok((
        i,
        AttributeInfo {
            attribute_name_index,
            attribute_length,
            info: info.as_range(),
        },
    ))
}

pub fn exception_entry_parser(input: ParseData) -> IResult<ParseData, ExceptionEntry> {
    do_parse!(
        input,
        start_pc: be_u16
            >> end_pc: be_u16
            >> handler_pc: be_u16
            >> catch_type: constant_pool_index_raw
            >> (ExceptionEntry {
                start_pc: InstructionIndex(start_pc),
                end_pc: InstructionIndex(end_pc),
                handler_pc: InstructionIndex(handler_pc),
                catch_type,
            })
    )
}

pub fn code_attribute_parser(i: ParseData) -> IResult<ParseData, CodeAttribute> {
    let (i, max_stack) = be_u16(i)?;
    let (i, max_locals) = be_u16(i)?;

    let (i, code_length) = be_u32(i)?;
    let (i, code) = take(code_length)(i)?;

    let (i, exception_table_length) = be_u16(i)?;
    let (i, exception_table) =
        count_sv(exception_entry_parser, usize::from(exception_table_length))(i)?;

    let (i, attributes_count) = be_u16(i)?;
    let (i, attributes) = count_sv(attribute_parser, usize::from(attributes_count))(i)?;

    Ok((
        i,
        CodeAttribute {
            max_stack,
            max_locals,
            code_length,
            code: code.as_range(),
            exception_table_length,
            exception_table,
            attributes_count,
            attributes,
        },
    ))
}

pub fn code_attribute_opt_parser(i: ParseData) -> IResult<ParseData, CodeAttributeOpt> {
    let (i, max_stack) = be_u16(i)?;
    let (i, max_locals) = be_u16(i)?;

    let (i, code_length) = be_u32(i)?;
    let code_start = i.pos();
    let (i, _code) = take(code_length)(i)?;
    let code_range = code_start..(code_start + code_length as usize);

    let (i, exception_table_length) = be_u16(i)?;
    let exception_table_start = i.pos();
    let (i, _) = skip_count(exception_entry_parser, usize::from(exception_table_length))(i)?;

    let (i, attributes_count) = be_u16(i)?;
    let attributes_start = i.pos();
    let (i, _) = skip_count(skip_attribute_parser, usize::from(attributes_count))(i)?;

    Ok((
        i,
        CodeAttributeOpt {
            max_stack,
            max_locals,
            code_range,
            exception_table_length,
            exception_table_start,
            attributes_count,
            attributes_start,
        },
    ))
}

fn same_frame_parser(input: ParseData, frame_type: u8) -> IResult<ParseData, StackMapFrame> {
    value!(input, SameFrame { frame_type })
}

fn verification_type_parser(input: ParseData) -> IResult<ParseData, VerificationTypeInfo> {
    use self::VerificationTypeInfo::*;
    let v = input.data()[0];
    let new_input = input.slice(1..);
    match v {
        0 => Ok((new_input, Top)),
        1 => Ok((new_input, Integer)),
        2 => Ok((new_input, Float)),
        3 => Ok((new_input, Double)),
        4 => Ok((new_input, Long)),
        5 => Ok((new_input, Null)),
        6 => Ok((new_input, UninitializedThis)),
        7 => do_parse!(
            new_input,
            class: constant_pool_index_raw >> (Object { class })
        ),
        8 => do_parse!(new_input, offset: be_u16 >> (Uninitialized { offset })),
        _ => Result::Err(Err::Error(nom::error::Error::new(input, ErrorKind::Alt))),
    }
}

fn same_locals_1_stack_item_frame_parser(
    input: ParseData,
    frame_type: u8,
) -> IResult<ParseData, StackMapFrame> {
    do_parse!(
        input,
        stack: verification_type_parser >> (SameLocals1StackItemFrame { frame_type, stack })
    )
}

fn same_locals_1_stack_item_frame_extended_parser(
    input: ParseData,
    frame_type: u8,
) -> IResult<ParseData, StackMapFrame> {
    do_parse!(
        input,
        offset_delta: be_u16
            >> stack: verification_type_parser
            >> (SameLocals1StackItemFrameExtended {
                frame_type,
                offset_delta,
                stack
            })
    )
}

fn chop_frame_parser(input: ParseData, frame_type: u8) -> IResult<ParseData, StackMapFrame> {
    do_parse!(
        input,
        offset_delta: be_u16
            >> (ChopFrame {
                frame_type,
                offset_delta
            })
    )
}

fn same_frame_extended_parser(
    input: ParseData,
    frame_type: u8,
) -> IResult<ParseData, StackMapFrame> {
    do_parse!(
        input,
        offset_delta: be_u16
            >> (SameFrameExtended {
                frame_type,
                offset_delta
            })
    )
}

fn append_frame_parser(i: ParseData, frame_type: u8) -> IResult<ParseData, StackMapFrame> {
    let (i, offset_delta) = be_u16(i)?;
    let (i, locals) = count_sv(verification_type_parser, (frame_type - 251) as usize)(i)?;
    Ok((
        i,
        AppendFrame {
            frame_type,
            offset_delta,
            locals,
        },
    ))
}

fn full_frame_parser(i: ParseData, frame_type: u8) -> IResult<ParseData, StackMapFrame> {
    let (i, offset_delta) = be_u16(i)?;
    let (i, number_of_locals) = be_u16(i)?;
    let (i, locals) = count_sv(verification_type_parser, number_of_locals as usize)(i)?;
    let (i, number_of_stack_items) = be_u16(i)?;
    let (i, stack) = count_sv(verification_type_parser, number_of_stack_items as usize)(i)?;
    Ok((
        i,
        FullFrame {
            frame_type,
            offset_delta,
            number_of_locals,
            locals,
            number_of_stack_items,
            stack,
        },
    ))
}

fn stack_frame_parser(input: ParseData, frame_type: u8) -> IResult<ParseData, StackMapFrame> {
    match frame_type {
        0..=63 => same_frame_parser(input, frame_type),
        64..=127 => same_locals_1_stack_item_frame_parser(input, frame_type),
        247 => same_locals_1_stack_item_frame_extended_parser(input, frame_type),
        248..=250 => chop_frame_parser(input, frame_type),
        251 => same_frame_extended_parser(input, frame_type),
        252..=254 => append_frame_parser(input, frame_type),
        255 => full_frame_parser(input, frame_type),
        _ => Result::Err(Err::Error(nom::error::Error::new(input, ErrorKind::Alt))),
    }
}

fn stack_map_frame_entry_parser(i: ParseData) -> IResult<ParseData, StackMapFrame> {
    let (i, frame_type) = be_u8(i)?;
    stack_frame_parser(i, frame_type)
}

pub fn stack_map_table_attribute_parser(
    input: ParseData,
) -> IResult<ParseData, StackMapTableAttribute> {
    do_parse!(
        input,
        number_of_entries: be_u16
            >> entries: count!(stack_map_frame_entry_parser, number_of_entries as usize)
            >> (StackMapTableAttribute {
                number_of_entries,
                entries,
            })
    )
}

pub fn exceptions_attribute_parser(input: ParseData) -> IResult<ParseData, ExceptionsAttribute> {
    do_parse!(
        input,
        exception_table_length: be_u16
            >> exception_table: count!(constant_pool_index_raw, exception_table_length as usize)
            >> (ExceptionsAttribute {
                exception_table_length,
                exception_table
            })
    )
}

pub fn constant_value_attribute_parser(
    input: ParseData,
) -> IResult<ParseData, ConstantValueAttribute> {
    do_parse!(
        input,
        constant_value_index: be_u16
            >> (ConstantValueAttribute {
                constant_value_index,
            })
    )
}

fn bootstrap_method_parser(input: ParseData) -> IResult<ParseData, BootstrapMethod> {
    do_parse!(
        input,
        bootstrap_method_ref: be_u16
            >> num_bootstrap_arguments: be_u16
            >> bootstrap_arguments: count!(be_u16, num_bootstrap_arguments as usize)
            >> (BootstrapMethod {
                bootstrap_method_ref,
                num_bootstrap_arguments,
                bootstrap_arguments,
            })
    )
}

pub fn bootstrap_methods_attribute_parser(
    input: ParseData,
) -> IResult<ParseData, BootstrapMethodsAttribute> {
    do_parse!(
        input,
        num_bootstrap_methods: be_u16
            >> bootstrap_methods: count!(bootstrap_method_parser, num_bootstrap_methods as usize)
            >> (BootstrapMethodsAttribute {
                num_bootstrap_methods,
                bootstrap_methods,
            })
    )
}

pub fn sourcefile_attribute_parser(input: ParseData) -> IResult<ParseData, SourceFileAttribute> {
    do_parse!(
        input,
        attribute_name_index: be_u16
            >> attribute_length: be_u32
            >> sourcefile_index: constant_pool_index_raw
            >> (SourceFileAttribute {
                attribute_name_index,
                attribute_length,
                sourcefile_index
            })
    )
}
