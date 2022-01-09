use std::iter::{Copied, Enumerate};
use std::ops::{Range, RangeFrom};
use std::slice::Iter;

use nom::bytes::complete::tag;
use nom::number::complete::be_u16;
use nom::{
    AsBytes, ExtendInto, FindSubstring, FindToken, IResult, InputIter, InputLength, InputTake,
    Needed, Slice, UnspecializedInput,
};

use crate::attribute_info::attribute_parser;
use crate::constant_info::constant_parser;
use crate::field_info::field_parser;
use crate::method_info::method_parser;
use crate::types::{ClassAccessFlags, ClassFile};
use crate::ClassFileVersion;

use crate::constant_pool::ConstantPool;
use crate::util::{constant_pool_index_raw, count_sv};

// named!(magic_parser, tag!(&[0xCA, 0xFE, 0xBA, 0xBE]));

fn magic_parser(i: ParseData) -> IResult<ParseData, ()> {
    let magic: &[u8] = &[0xCA, 0xFE, 0xBA, 0xBE];
    let (i, _) = tag(magic)(i)?;
    Ok((i, ()))
}

/// Parse a byte array into a ClassFile. This will probably be deprecated in 0.4.0 in as it returns
/// a nom IResult type, which exposes the internal parsing library and not a good idea.
///
/// If you want to call it directly, as it is the only way to parse a byte slice directly, you must
/// unwrap the result yourself.
///
/// ```rust
/// # use classfile_parser::parser::ParseData;
/// let classfile_bytes: &[u8] = include_bytes!("../java-assets/compiled-classes/BasicClass.class");
///
/// match classfile_parser::class_parser(ParseData::new(classfile_bytes)) {
///     Ok((_, class_file)) => {
///         println!("version {:?}", class_file.version);
///     }
///     Err(_) => panic!("Failed to parse"),
/// };
/// ```
pub fn class_parser(i: ParseData) -> IResult<ParseData, ClassFile> {
    let (i, _) = magic_parser(i)?;

    let (i, minor_version) = be_u16(i)?;
    let (i, major_version) = be_u16(i)?;

    let (i, const_pool_size) = be_u16(i)?;
    let (i, const_pool) = constant_parser(i, (const_pool_size - 1).into())?;

    let (i, access_flags) = be_u16(i)?;

    let (i, this_class) = constant_pool_index_raw(i)?;
    let (i, super_class) = constant_pool_index_raw(i)?;

    let (i, interfaces_count) = be_u16(i)?;
    let (i, interfaces) = count_sv(constant_pool_index_raw, interfaces_count.into())(i)?;

    let (i, fields_count) = be_u16(i)?;
    let (i, fields) = count_sv(field_parser, fields_count.into())(i)?;

    let (i, methods_count) = be_u16(i)?;
    let (i, methods) = count_sv(method_parser, methods_count.into())(i)?;

    let (i, attributes_count) = be_u16(i)?;
    let (i, attributes) = count_sv(attribute_parser, attributes_count.into())(i)?;

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
            this_class,
            super_class,
            interfaces_count,
            interfaces,
            fields_count,
            fields,
            methods_count,
            methods,
            attributes_count,
            attributes,
        },
    ))
}

/// Keeps track of the current slice and the position within the actual data that we're at
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseData<'a> {
    /// This data is already sliced
    data: &'a [u8],
    /// The position within the outermost data that the slice starts at
    pos: usize,
}
impl<'a> ParseData<'a> {
    pub fn new(data: &'a [u8]) -> ParseData<'a> {
        ParseData { data, pos: 0 }
    }

    /// Create ParseData where the data is the entirety of it and the Range is a range created
    /// from it in the past.
    pub fn from_range(data: &'a [u8], range: Range<usize>) -> ParseData<'a> {
        let pos = range.start;
        let data = &data[range];
        ParseData { data, pos }
    }

    pub fn data(&self) -> &'a [u8] {
        self.data
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn pos(&self) -> usize {
        self.pos
    }

    pub fn as_range(&self) -> Range<usize> {
        self.pos..(self.pos + self.data.len())
    }
}
impl<'a> AsBytes for ParseData<'a> {
    fn as_bytes(&self) -> &[u8] {
        self.data
    }
}
// impl<'a> Compare<ParseData<'a>> for ParseData<'a> {
//     fn compare(&self, t: ParseData<'a>) -> nom::CompareResult {
//         if self.data == t.data && self.pos == t.pos {
//             nom::CompareResult::Ok
//         } else {
//             nom::CompareResult::Error
//         }
//     }

//     fn compare_no_case(&self, t: ParseData<'a>) -> nom::CompareResult {
//         self.compare(t)
//     }
// }
// impl<'a, 'b> Compare<&'b [u8]> for ParseData<'a> {
//     fn compare(&self, t: &'b [u8]) -> nom::CompareResult {
//         if self.data == t {
//             nom::CompareResult::Ok
//         } else {
//             nom::CompareResult::Error
//         }
//     }

//     fn compare_no_case(&self, t: &'b [u8]) -> nom::CompareResult {
//         if self.data == t {
//             nom::CompareResult::Ok
//         } else {
//             nom::CompareResult::Error
//         }
//     }
// }
impl<'a> ExtendInto for ParseData<'a> {
    type Item = u8;

    type Extender = Vec<u8>;

    fn new_builder(&self) -> Self::Extender {
        Vec::new()
    }

    fn extend_into(&self, acc: &mut Self::Extender) {
        acc.extend_from_slice(self.data)
    }
}
impl<'a> FindToken<u8> for ParseData<'a> {
    fn find_token(&self, token: u8) -> bool {
        self.data.find_token(token)
    }
}
impl<'a, 'b> FindToken<&'b u8> for ParseData<'a> {
    fn find_token(&self, token: &'b u8) -> bool {
        self.data.find_token(token)
    }
}
impl<'a, 'b> FindSubstring<&'b [u8]> for ParseData<'a> {
    fn find_substring(&self, substr: &'b [u8]) -> Option<usize> {
        self.data.find_substring(substr)
    }
}
impl<'a> Slice<RangeFrom<usize>> for ParseData<'a> {
    fn slice(&self, range: RangeFrom<usize>) -> Self {
        ParseData {
            data: &self.data[range.clone()],
            pos: self.pos + range.start,
        }
    }
}
impl<'a> InputLength for ParseData<'a> {
    fn input_len(&self) -> usize {
        self.data.len()
    }
}
impl<'a> InputIter for ParseData<'a> {
    type Item = u8;

    type Iter = Enumerate<Self::IterElem>;

    type IterElem = Copied<Iter<'a, u8>>;

    fn iter_indices(&self) -> Self::Iter {
        self.iter_elements().enumerate()
    }

    fn iter_elements(&self) -> Self::IterElem {
        self.data.iter().copied()
    }

    fn position<P>(&self, predicate: P) -> Option<usize>
    where
        P: Fn(Self::Item) -> bool,
    {
        self.data.iter().position(|b| predicate(*b))
    }

    fn slice_index(&self, count: usize) -> Result<usize, nom::Needed> {
        if self.data.len() >= count {
            Ok(count)
        } else {
            Err(Needed::new(count - self.data.len()))
        }
    }
}
impl<'a> InputTake for ParseData<'a> {
    fn take(&self, count: usize) -> Self {
        ParseData {
            data: &self.data[0..count],
            pos: self.pos,
        }
    }

    fn take_split(&self, count: usize) -> (Self, Self) {
        let (prefix, suffix) = self.data.split_at(count);
        (
            // Data that they can continue to parsing
            ParseData {
                data: suffix,
                pos: self.pos + count,
            },
            // The data to use
            ParseData {
                data: prefix,
                pos: self.pos,
            },
        )
    }
}
impl<'a> UnspecializedInput for ParseData<'a> {}
