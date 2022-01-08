use nom::bytes::complete::tag;
use nom::multi::count;
use nom::number::complete::be_u16;
use nom::IResult;

use crate::attribute_info::attribute_parser;
use crate::constant_info::constant_parser;
use crate::field_info::field_parser;
use crate::method_info::method_parser;
use crate::types::{ClassAccessFlags, ClassFile};
use crate::ClassFileVersion;

use crate::constant_pool::{ConstantPool, ConstantPoolIndexRaw};

// named!(magic_parser, tag!(&[0xCA, 0xFE, 0xBA, 0xBE]));

fn magic_parser(i: &[u8]) -> IResult<&[u8], ()> {
    let (i, _) = tag(&[0xCA, 0xFE, 0xBA, 0xBE])(i)?;
    Ok((i, ()))
}

/// Parse a byte array into a ClassFile. This will probably be deprecated in 0.4.0 in as it returns
/// a nom IResult type, which exposes the internal parsing library and not a good idea.
///
/// If you want to call it directly, as it is the only way to parse a byte slice directly, you must
/// unwrap the result yourself.
///
/// ```rust
/// let classfile_bytes = include_bytes!("../java-assets/compiled-classes/BasicClass.class");
///
/// match classfile_parser::class_parser(classfile_bytes) {
///     Ok((_, class_file)) => {
///         println!("version {:?}", class_file.version);
///     }
///     Err(_) => panic!("Failed to parse"),
/// };
/// ```
pub fn class_parser(i: &[u8]) -> IResult<&[u8], ClassFile> {
    let (i, _) = magic_parser(i)?;

    let (i, minor_version) = be_u16(i)?;
    let (i, major_version) = be_u16(i)?;

    let (i, const_pool_size) = be_u16(i)?;
    let (i, const_pool) = constant_parser(i, (const_pool_size - 1).into())?;

    let (i, access_flags) = be_u16(i)?;

    let (i, this_class) = be_u16(i)?;
    let (i, super_class) = be_u16(i)?;

    let (i, interfaces_count) = be_u16(i)?;
    let (i, interfaces) = count(be_u16, interfaces_count.into())(i)?;

    let (i, fields_count) = be_u16(i)?;
    let (i, fields) = count(field_parser, fields_count.into())(i)?;

    let (i, methods_count) = be_u16(i)?;
    let (i, methods) = count(method_parser, methods_count.into())(i)?;

    let (i, attributes_count) = be_u16(i)?;
    let (i, attributes) = count(attribute_parser, attributes_count.into())(i)?;

    Ok((
        i,
        ClassFile {
            version: ClassFileVersion {
                major: major_version,
                minor: minor_version,
            },
            const_pool_size,
            const_pool: ConstantPool::new(const_pool),
            access_flags: ClassAccessFlags::from_bits_truncate(access_flags),
            this_class: ConstantPoolIndexRaw::new(this_class),
            super_class: ConstantPoolIndexRaw::new(super_class),
            interfaces_count,
            interfaces: interfaces
                .into_iter()
                .map(ConstantPoolIndexRaw::new)
                .collect(),
            fields_count,
            fields,
            methods_count,
            methods,
            attributes_count,
            attributes,
        },
    ))
}
