use nom::number::complete::be_u16;
use nom::IResult;

use crate::attribute_info::attribute_parser;

use crate::field_info::{FieldAccessFlags, FieldInfo};

use crate::util::{constant_pool_index_raw, count_sv};

pub fn field_parser(i: &[u8]) -> IResult<&[u8], FieldInfo> {
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
