use nom::{be_u16, IResult};

use crate::attribute_info::attribute_parser;

use crate::method_info::{MethodAccessFlags, MethodInfo};

use crate::constant_pool::ConstantPoolIndexRaw;

pub fn method_parser(input: &[u8]) -> IResult<&[u8], MethodInfo> {
    do_parse!(
        input,
        access_flags: be_u16
            >> name_index: be_u16
            >> descriptor_index: be_u16
            >> attributes_count: be_u16
            >> attributes: count!(attribute_parser, attributes_count as usize)
            >> (MethodInfo {
                access_flags: MethodAccessFlags::from_bits_truncate(access_flags),
                name_index: ConstantPoolIndexRaw::new(name_index),
                descriptor_index: ConstantPoolIndexRaw::new(descriptor_index),
                attributes_count,
                attributes,
            })
    )
}
