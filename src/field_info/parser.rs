use nom::number::complete::be_u16;
use nom::IResult;

use crate::attribute_info::{attribute_parser, skip_attribute_parser, constant_value_attribute_parser};

use crate::constant_info::ConstantInfo;
use crate::constant_pool::{ConstantPoolIndexRaw, ConstantPool};
use crate::field_info::{FieldAccessFlags, FieldInfo};

use crate::method_info::attributes_search_parser;
use crate::parser::ParseData;
use crate::util::{constant_pool_index_raw, count_sv, skip_count};

use super::FieldInfoOpt;

pub fn skip_field_parser(i: ParseData) -> IResult<ParseData, ()> {
    let (i, _) = be_u16(i)?;
    let (i, _) = constant_pool_index_raw::<ConstantInfo>(i)?;
    let (i, _) = constant_pool_index_raw::<ConstantInfo>(i)?;
    let (i, attributes_count) = be_u16(i)?;
    let (i, _) = skip_count(skip_attribute_parser, attributes_count.into())(i)?;
    Ok((i, ()))
}

pub fn field_parser(i: ParseData) -> IResult<ParseData, FieldInfo> {
    let (i, access_flags) = be_u16(i)?;
    let (i, name_index) = constant_pool_index_raw(i)?;
    let (i, descriptor_index) = constant_pool_index_raw(i)?;
    let (i, attributes_count) = be_u16(i)?;
    let (i, attributes) = count_sv(attribute_parser, attributes_count.into())(i)?;
    Ok((
        i,
        FieldInfo {
            access_flags: FieldAccessFlags::from_bits_truncate(access_flags),
            name_index,
            descriptor_index,
            attributes_count,
            attributes,
        },
    ))
}


pub fn field_opt_parser(i: ParseData) -> IResult<ParseData, FieldInfoOpt> {
    let (i, access_flags) = be_u16(i)?;
    let (i, name_index) = constant_pool_index_raw(i)?;
    let (i, descriptor_index) = constant_pool_index_raw(i)?;
    let (i, attributes_count) = be_u16(i)?;
    let (i, _) = skip_count(skip_attribute_parser, attributes_count.into())(i)?;
    Ok((
        i,
        FieldInfoOpt {
            access_flags: FieldAccessFlags::from_bits_truncate(access_flags),
            name_index,
            descriptor_index,
            attributes_count,
        },
    ))
}

/// Parse the field opt and search for a constant value initializer, returning the index to that as
/// well, if it exists.
pub fn field_opt_value_parser<'a>(i: ParseData<'a>, class_file_data: &'a [u8], constant_pool: &ConstantPool) -> IResult<ParseData<'a>, (FieldInfoOpt, Option<ConstantPoolIndexRaw<ConstantInfo>>)> {
    let (i, access_flags) = be_u16(i)?;
    let (i, name_index) = constant_pool_index_raw(i)?;
    let (i, descriptor_index) = constant_pool_index_raw(i)?;
    let (i, attributes_count) = be_u16(i)?;
    let before_attr_i = i.clone();
    let (_, attr) = attributes_search_parser(i, class_file_data, constant_pool, "ConstantValue", attributes_count)?;

    let attr = if let Some((_, info_range)) = attr {
        let i = ParseData::from_range(class_file_data, info_range);
        let (_, attr) = constant_value_attribute_parser(i)?;
        Some(attr.constant_value_index)
    } else {
        None
    };

    // TODO: We could do better, since after searching through attributes we could know 
    // how far along we got, and then continue from there.
    let (i, _) = skip_count(skip_attribute_parser, attributes_count.into())(before_attr_i)?;
    Ok((
        i,
        (FieldInfoOpt {
            access_flags: FieldAccessFlags::from_bits_truncate(access_flags),
            name_index,
            descriptor_index,
            attributes_count,
        }, attr),
    ))
}