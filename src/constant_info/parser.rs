use nom::error::ErrorKind;
use nom::number::complete::{be_f32, be_f64, be_i32, be_i64, be_u16, be_u8};
use nom::{Err, IResult};

use crate::constant_info::*;
use crate::parser::ParseData;
use crate::util::constant_pool_index_raw;

named!(const_utf8<ParseData, ConstantInfo>, do_parse!(
    length: be_u16 >>
    bytes: take!(length) >>
    (ConstantInfo::Utf8(Utf8Constant::new(bytes.as_range())))
));

named!(const_integer<ParseData, ConstantInfo>, do_parse!(
    value: be_i32 >>
    (ConstantInfo::Integer(
        IntegerConstant {
            value,
        }
    ))
));

named!(const_float<ParseData, ConstantInfo>, do_parse!(
    value: be_f32 >>
    (ConstantInfo::Float(
        FloatConstant {
            value,
        }
    ))
));

named!(const_long<ParseData, ConstantInfo>, do_parse!(
    value: be_i64 >>
    (ConstantInfo::Long(
        LongConstant {
            value,
        }
    ))
));

named!(const_double<ParseData, ConstantInfo>, do_parse!(
    value: be_f64 >>
    (ConstantInfo::Double(
        DoubleConstant {
            value,
        }
    ))
));

named!(const_class<ParseData, ConstantInfo>, do_parse!(
    name_index: constant_pool_index_raw >>
    (ConstantInfo::Class(
        ClassConstant {
            name_index,
        }
    ))
));

named!(const_string<ParseData, ConstantInfo>, do_parse!(
    string_index: constant_pool_index_raw >>
    (ConstantInfo::String(
        StringConstant {
            string_index,
        }
    ))
));

named!(const_field_ref<ParseData, ConstantInfo>, do_parse!(
    class_index: constant_pool_index_raw >>
    name_and_type_index: constant_pool_index_raw >>
    (ConstantInfo::FieldRef(
        FieldRefConstant {
            class_index,
            name_and_type_index,
        }
    ))
));

named!(const_method_ref<ParseData, ConstantInfo>, do_parse!(
    class_index: constant_pool_index_raw >>
    name_and_type_index: constant_pool_index_raw >>
    (ConstantInfo::MethodRef(
        MethodRefConstant {
            class_index,
            name_and_type_index,
        }
    ))
));

named!(const_interface_method_ref<ParseData, ConstantInfo>, do_parse!(
    class_index: constant_pool_index_raw >>
    name_and_type_index: constant_pool_index_raw >>
    (ConstantInfo::InterfaceMethodRef(
        InterfaceMethodRefConstant {
            class_index,
            name_and_type_index,
        }
    ))
));

named!(const_name_and_type<ParseData, ConstantInfo>, do_parse!(
    name_index: constant_pool_index_raw >>
    descriptor_index: constant_pool_index_raw >>
    (ConstantInfo::NameAndType(
        NameAndTypeConstant {
            name_index,
            descriptor_index,
        }
    ))
));

named!(const_method_handle<ParseData, ConstantInfo>, do_parse!(
    reference_kind: be_u8 >>
    reference_index: constant_pool_index_raw >>
    (ConstantInfo::MethodHandle(
        MethodHandleConstant {
            reference_kind,
            reference_index,
        }
    ))
));

named!(const_method_type<ParseData, ConstantInfo>, do_parse!(
    descriptor_index: constant_pool_index_raw >>
    (ConstantInfo::MethodType(
        MethodTypeConstant {
            descriptor_index,
        }
    ))
));

named!(const_invoke_dynamic<ParseData, ConstantInfo>, do_parse!(
    bootstrap_method_attr_index: be_u16 >>
    name_and_type_index: constant_pool_index_raw >>
    (ConstantInfo::InvokeDynamic(
        InvokeDynamicConstant {
            bootstrap_method_attr_index,
            name_and_type_index,
        }
    ))
));

fn const_block_parser(input: ParseData, const_type: u8) -> IResult<ParseData, ConstantInfo> {
    match const_type {
        1 => const_utf8(input),
        3 => const_integer(input),
        4 => const_float(input),
        5 => const_long(input),
        6 => const_double(input),
        7 => const_class(input),
        8 => const_string(input),
        9 => const_field_ref(input),
        10 => const_method_ref(input),
        11 => const_interface_method_ref(input),
        12 => const_name_and_type(input),
        15 => const_method_handle(input),
        16 => const_method_type(input),
        18 => const_invoke_dynamic(input),
        _ => Result::Err(Err::Error(error_position!(input, ErrorKind::Alt))),
    }
}

fn single_constant_parser(i: ParseData) -> IResult<ParseData, ConstantInfo> {
    let (i, const_type) = be_u8(i)?;
    let (i, const_block) = const_block_parser(i, const_type)?;
    Ok((i, const_block))
}

pub fn constant_parser(
    i: ParseData,
    const_pool_size: usize,
) -> IResult<ParseData, Vec<ConstantInfo>> {
    let mut index = 0;
    let mut input = i;
    let mut res = Vec::with_capacity(const_pool_size);
    while index < const_pool_size {
        match single_constant_parser(input.clone()) {
            Ok((i, o)) => {
                // Long and Double Entries have twice the size
                // see https://docs.oracle.com/javase/specs/jvms/se6/html/ClassFile.doc.html#1348
                let uses_two_entries =
                    matches!(o, ConstantInfo::Long(..) | ConstantInfo::Double(..));

                res.push(o);
                if uses_two_entries {
                    res.push(ConstantInfo::Unusable);
                    index += 1;
                }
                input = i;
                index += 1;
            }
            _ => {
                return Result::Err(Err::Error(nom::error::Error::new(input, ErrorKind::Alt)));
            }
        }
    }
    Ok((input, res))
}
