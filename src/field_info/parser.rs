use nom::{be_u16, IResult};

use attribute_info::attribute_parser;

use field_info::{FieldAccessFlags, FieldInfo};

use crate::ConstantPoolIndexRaw;

pub fn field_parser(input: &[u8]) -> IResult<&[u8], FieldInfo> {
    do_parse!(
        input,
        access_flags: be_u16
            >> name_index: be_u16
            >> descriptor_index: be_u16
            >> attributes_count: be_u16
            >> attributes: count!(attribute_parser, attributes_count as usize)
            >> (FieldInfo {
                access_flags: FieldAccessFlags::from_bits_truncate(access_flags),
                name_index: ConstantPoolIndexRaw::new(name_index),
                descriptor_index: ConstantPoolIndexRaw::new(descriptor_index),
                attributes_count,
                attributes,
            })
    )
}
