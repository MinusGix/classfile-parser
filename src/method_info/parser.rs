use nom::multi::count;
use nom::number::complete::be_u16;
use nom::IResult;

use crate::attribute_info::attribute_parser;

use crate::method_info::{MethodAccessFlags, MethodInfo};

use crate::util::constant_pool_index_raw;

pub fn method_parser(i: &[u8]) -> IResult<&[u8], MethodInfo> {
    let (i, access_flags) = be_u16(i)?;
    let (i, name_index) = constant_pool_index_raw(i)?;
    let (i, descriptor_index) = constant_pool_index_raw(i)?;
    let (i, attributes_count) = be_u16(i)?;
    let (i, attributes) = count(attribute_parser, attributes_count.into())(i)?;
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
