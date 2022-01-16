use std::borrow::Cow;
use std::ops::Range;

use nom::bytes::complete::take;

use nom::number::complete::{be_u16, be_u32};
use nom::IResult;

use crate::attribute_info::{attribute_parser, skip_attribute_parser};

use crate::constant_info::{ConstantInfo, Utf8Constant};
use crate::constant_pool::ConstantPool;
use crate::method_info::{MethodAccessFlags, MethodInfo};

use crate::parser::ParseData;
use crate::util::{constant_pool_index_raw, count_sv, skip_count};

use super::MethodInfoOpt;

pub fn skip_method_parser(i: ParseData) -> IResult<ParseData, ()> {
    let (i, _) = be_u16(i)?;
    let (i, _) = constant_pool_index_raw::<ConstantInfo>(i)?;
    let (i, _) = constant_pool_index_raw::<ConstantInfo>(i)?;
    let (i, attributes_count) = be_u16(i)?;
    let (i, _) = skip_count(skip_attribute_parser, attributes_count.into())(i)?;
    Ok((i, ()))
}

pub fn skip_method_attributes_parser(
    i: ParseData,
    attributes_count: u16,
) -> IResult<ParseData, ()> {
    let (i, _) = skip_count(skip_attribute_parser, attributes_count.into())(i)?;
    Ok((i, ()))
}

pub fn method_parser(i: ParseData) -> IResult<ParseData, MethodInfo> {
    let (i, access_flags) = be_u16(i)?;
    let (i, name_index) = constant_pool_index_raw(i)?;
    let (i, descriptor_index) = constant_pool_index_raw(i)?;
    let (i, attributes_count) = be_u16(i)?;
    let (i, attributes) = count_sv(attribute_parser, attributes_count.into())(i)?;
    Ok((
        i,
        MethodInfo {
            access_flags: MethodAccessFlags::from_bits_truncate(access_flags),
            name_index,
            descriptor_index,
            attributes_count,
            attributes,
        },
    ))
}

pub fn method_opt_parser(i: ParseData) -> IResult<ParseData, MethodInfoOpt> {
    let (i, access_flags) = be_u16(i)?;
    let (i, name_index) = constant_pool_index_raw(i)?;
    let (i, descriptor_index) = constant_pool_index_raw(i)?;
    let (i, attributes_count) = be_u16(i)?;

    Ok((
        i,
        MethodInfoOpt {
            access_flags: MethodAccessFlags::from_bits_truncate(access_flags),
            name_index,
            descriptor_index,
            attributes_count,
        },
    ))
}

// TODO: This method could probably call out to the attribute parser mod instead, so that we don't
// duplicate the code as much
/// This returns the range for the given attributes data
pub fn attributes_search_parser<'a>(
    input: ParseData<'a>,
    class_file_data: &[u8],
    constant_pool: &ConstantPool,
    name: &str,
    attributes_count: u16,
) -> IResult<ParseData<'a>, Option<Range<usize>>> {
    let mut input = input;
    for _ in 0..attributes_count {
        let (i, name_index) = constant_pool_index_raw::<Utf8Constant>(input)?;
        if let Some(attr_name) = constant_pool.get_t(name_index) {
            let attr_name = attr_name.as_text(class_file_data);
            if Cow::Borrowed(name) == attr_name {
                let (i, attribute_length) = be_u32(i)?;
                let (i, info) = take(attribute_length)(i)?;
                return Ok((i, Some(info.as_range())));
            }
        }
        let (i, attribute_length) = be_u32(i)?;
        let (i, _) = take(attribute_length)(i)?;
        input = i;
    }

    Ok((input, None))
}
