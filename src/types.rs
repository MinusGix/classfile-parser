use std::borrow::Cow;
use std::ops::Range;

use nom::number::complete::be_u16;
use smallvec::SmallVec;

use crate::attribute_info::AttributeInfo;
use crate::constant_info::ConstantInfo;
use crate::field_info::{field_opt_value_parser, FieldInfo, FieldInfoOpt};
use crate::method_info::{
    attributes_search_parser, method_opt_parser, method_parser, skip_method_attributes_parser,
    skip_method_parser, MethodInfo, MethodInfoOpt,
};

use crate::parser::ParseData;
use crate::util::{constant_pool_index_raw, count_sv, skip_count};
use crate::{
    constant_info::ClassConstant,
    constant_pool::{ConstantPool, ConstantPoolIndexRaw},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClassFileJavaVersion {
    /// The major version for 1.0.2 and 1.1 is the same, so unless there's
    /// specific observable differences, they appear the same.
    V1_1 = 45,
    V1_2 = 46,
    V1_3 = 47,
    V1_4 = 48,
    V5 = 49,
    V6 = 50,
    V7 = 51,
    V8 = 52,
    V9 = 53,
    V10 = 54,
    V11 = 55,
    V12 = 56,
    V13 = 57,
}
impl ClassFileJavaVersion {
    pub fn from_version(major_version: u16, _minor_version: u16) -> Option<ClassFileJavaVersion> {
        Some(match major_version {
            45 => Self::V1_1,
            46 => Self::V1_2,
            47 => Self::V1_3,
            48 => Self::V1_4,
            49 => Self::V5,
            50 => Self::V6,
            51 => Self::V7,
            52 => Self::V8,
            53 => Self::V9,
            54 => Self::V10,
            55 => Self::V11,
            56 => Self::V12,
            57 => Self::V13,
            _ => return None,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClassFileVersion {
    pub major: u16,
    pub minor: u16,
}
impl ClassFileVersion {
    pub fn into_java_version(self) -> Option<ClassFileJavaVersion> {
        ClassFileJavaVersion::from_version(self.major, self.minor)
    }
}

bitflags! {
    pub struct ClassAccessFlags: u16 {
        const PUBLIC = 0x0001;     //	Declared public; may be accessed from outside its package.
        const FINAL = 0x0010;      //	Declared final; no subclasses allowed.
        const SUPER = 0x0020;      //	Treat superclass methods specially when invoked by the invokespecial instruction.
        const INTERFACE = 0x0200;  //	Is an interface, not a class.
        const ABSTRACT = 0x0400;   //	Declared abstract; must not be instantiated.
        const SYNTHETIC = 0x1000;  //	Declared synthetic; not present in the source code.
        const ANNOTATION = 0x2000; //	Declared as an annotation type.
        const ENUM = 0x4000;       //	Declared as an enum type.
    }
}

/// An error in loading data
#[derive(Debug, Clone)]
pub enum LoadError {
    /// Some unknown error
    Unknown,
}

#[derive(Clone, Debug)]
pub struct ClassFile {
    pub version: ClassFileVersion,
    pub const_pool_size: u16,
    pub const_pool: ConstantPool,
    pub access_flags: ClassAccessFlags,
    pub this_class: ConstantPoolIndexRaw<ClassConstant>,
    pub super_class: ConstantPoolIndexRaw<ClassConstant>,
    pub interfaces_count: u16,
    pub interfaces: SmallVec<[ConstantPoolIndexRaw<ClassConstant>; 4]>,
    pub fields_count: u16,
    pub fields: SmallVec<[FieldInfo; 6]>,
    pub methods_count: u16,
    pub methods: SmallVec<[MethodInfo; 6]>,
    pub attributes_count: u16,
    pub attributes: SmallVec<[AttributeInfo; 4]>,
}

#[derive(Clone, Debug)]
pub struct ClassFileOpt {
    pub version: ClassFileVersion,
    pub const_pool_size: u16,
    pub const_pool: ConstantPool,
    pub access_flags: ClassAccessFlags,
    pub this_class: ConstantPoolIndexRaw<ClassConstant>,
    pub super_class: ConstantPoolIndexRaw<ClassConstant>,
    pub interfaces_count: u16,
    pub interfaces: SmallVec<[ConstantPoolIndexRaw<ClassConstant>; 4]>,
    pub fields: OptSmallVec<FieldInfo, 6>,
    pub methods: OptSmallVec<MethodInfo, 6>,
    pub attributes: OptSmallVec<AttributeInfo, 4>,
}
impl ClassFileOpt {
    // TODO: Return more useful errors

    pub fn load_attribute_with_name(
        &self,
        data: &[u8],
        name: &str,
    ) -> Result<Option<Range<usize>>, LoadError> {
        let input = ParseData::from_pos(data, self.attributes.start_pos);
        let (_, info) =
            attributes_search_parser(input, data, &self.const_pool, name, self.attributes.count)
                .map_err(|_| LoadError::Unknown)?;

        let info = info.map(|x| x.1);

        Ok(info)
    }

    /// Loads a method at a given index
    /// Returns the value in cache if there was one
    /// Returns an owned value if there wasn't, and does not insert into cache
    pub fn load_method_at(&self, data: &[u8], index: u16) -> Result<Cow<MethodInfo>, LoadError> {
        if !self.methods.contains_index(index) {
            return Err(LoadError::Unknown);
        }

        if let Some(method) = self.methods.get_opt(index) {
            return Ok(Cow::Borrowed(method));
        }

        let start_pos = self.methods.start_pos();
        let input = ParseData::from_pos(data, start_pos);
        let (input, _) = skip_count(skip_method_parser, usize::from(index))(input)
            .map_err(|_| LoadError::Unknown)?;

        method_parser(input)
            .map_err(|_| LoadError::Unknown)
            .map(|x| Cow::Owned(x.1))
    }

    /// Loads a method at a given index
    /// This returns the Opt version, which does not have attributes, which is cheaper
    /// Returns the value in cache if there was one
    /// Returns and owned value if there wasn't, and does not insert into cache
    /// It also returns the index of the data directly after it, aka the attributes count
    pub fn load_method_opt_at(&self, data: &[u8], index: u16) -> Result<MethodInfoOpt, LoadError> {
        if !self.methods.contains_index(index) {
            return Err(LoadError::Unknown);
        }

        if let Some(method) = self.methods.get_opt(index) {
            return Ok(MethodInfoOpt::from_method_info(method));
        }

        let start_pos = self.methods.start_pos();
        let input = ParseData::from_pos(data, start_pos);
        let (input, _) = skip_count(skip_method_parser, usize::from(index))(input)
            .map_err(|_| LoadError::Unknown)?;

        method_opt_parser(input)
            .map_err(|_| LoadError::Unknown)
            .map(|(_, method)| method)
    }

    /// This is guaranteed to be in order
    pub fn load_method_opt_iter<'a>(
        &'a self,
        data: &'a [u8],
    ) -> impl Iterator<Item = MethodInfoOpt> + 'a {
        let start_pos = self.methods.start_pos();
        if let Some(methods) = self.methods.data() {
            return MethodOptIter::Methods(methods);
        }

        let input = ParseData::from_pos(data, start_pos);
        MethodOptIter::Parse(input)
    }

    /// Does not load all methods if they're already loaded
    pub fn load_all_methods_mut(&mut self, data: &[u8]) -> Result<(), LoadError> {
        if self.methods.has_data() {
            return Ok(());
        }

        let start_pos = self.methods.start_pos();
        let input = ParseData::from_pos(data, start_pos);
        let (_, methods) = count_sv(method_parser, usize::from(self.methods.len()))(input)
            .map_err(|_| LoadError::Unknown)?;

        self.methods.fill(methods);

        Ok(())
    }

    /// Loads the method at the given index and tries to find an attribute, if it exists, with the
    /// given name
    pub fn load_method_attribute_info_at_with_name<'a>(
        &self,
        data: &'a [u8],
        index: u16,
        name: &str,
    ) -> Result<Option<Range<usize>>, LoadError> {
        let (attr_info_start, method) = {
            // TODO: This could do slightly better
            let start_pos = self.methods.start_pos();
            let input = ParseData::from_pos(data, start_pos);
            let (input, _) = skip_count(skip_method_parser, usize::from(index))(input)
                .map_err(|_| LoadError::Unknown)?;

            method_opt_parser(input)
                .ok()
                .map(|(i, method)| (i.pos(), method))
        }
        .ok_or(LoadError::Unknown)?;
        // TODO: make this for more general usage
        let input = ParseData::from_pos(data, attr_info_start);
        let (_, info) =
            attributes_search_parser(input, data, &self.const_pool, name, method.attributes_count)
                .map_err(|_| LoadError::Unknown)?;
        let info = info.map(|x| x.1);

        Ok(info)
    }

    // TODO: provide actual error type
    pub fn load_fields_values_iter<'a>(
        &'a self,
        data: &'a [u8],
    ) -> impl Iterator<
        Item = Result<(FieldInfoOpt, Option<ConstantPoolIndexRaw<ConstantInfo>>), LoadError>,
    > + 'a {
        let start_pos = self.fields.start_pos();
        let count = self.fields.count;
        // TODO: use cached data if it exists

        let mut p_input = ParseData::from_pos(data, start_pos);
        let mut done = false;
        let mut processed = 0;
        std::iter::from_fn(move || {
            if done {
                return None;
            } else if processed >= count {
                done = true;
                return None;
            }

            let i = p_input.clone();

            let (i, (field, value_index)) =
                if let Ok((i, f)) = field_opt_value_parser(i, data, &self.const_pool) {
                    (i, f)
                } else {
                    return Some(Err(LoadError::Unknown));
                };

            p_input = i;
            processed += 1;

            Some(Ok((field, value_index)))
        })
    }
}

enum MethodOptIter<'a, 'c> {
    Methods(&'a [MethodInfo]),
    Parse(ParseData<'c>),
    Error,
}
impl<'a, 'c> Iterator for MethodOptIter<'a, 'c> {
    type Item = MethodInfoOpt;

    // TODO: This code is kinda bad
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            MethodOptIter::Methods(methods) => {
                if let Some(method) = methods.first() {
                    *methods = &methods[1..];
                    Some(MethodInfoOpt::from_method_info(method))
                } else {
                    None
                }
            }
            MethodOptIter::Parse(parse) => {
                let input = parse.clone();
                if let Ok((input, method_opt)) = method_opt_parser(input) {
                    if let Ok((input, _)) =
                        skip_method_attributes_parser(input, method_opt.attributes_count)
                    {
                        *parse = input;
                        Some(method_opt)
                    } else {
                        *self = MethodOptIter::Error;
                        None
                    }
                } else {
                    *self = MethodOptIter::Error;
                    None
                }
            }
            MethodOptIter::Error => None,
        }
    }
}

/// A small vec that has content that may or may not exist, but includes the position it starts at
#[derive(Debug, Clone)]
pub struct OptSmallVec<T, const N: usize> {
    start_pos: usize,
    /// The number of elements that are expected, since most data has this already.
    count: u16,
    data: Option<SmallVec<[T; N]>>,
}
impl<T, const N: usize> OptSmallVec<T, N> {
    pub(crate) fn empty(start_pos: usize, count: u16) -> OptSmallVec<T, N> {
        OptSmallVec {
            start_pos,
            count,
            data: None,
        }
    }

    pub fn has_data(&self) -> bool {
        self.data.is_some()
    }

    pub fn data(&self) -> Option<&[T]> {
        self.data.as_ref().map(|x| x.as_slice())
    }

    pub fn fill(&mut self, data: SmallVec<[T; N]>) {
        self.data = Some(data);
    }

    /// The position that the date starts in the file
    pub fn start_pos(&self) -> usize {
        self.start_pos
    }

    /// Returns None if it wouldn't exist
    /// Also returns None if it isn't yet parsed
    pub fn get_opt(&self, index: u16) -> Option<&T> {
        self.data.as_ref().and_then(|x| x.get(usize::from(index)))
    }

    /// Note that this only tells you if it _would_ contain that index
    pub fn contains_index(&self, index: u16) -> bool {
        index < self.count
    }

    /// Note that this only tells you the number that _would_ exist
    pub fn len(&self) -> u16 {
        self.count
    }

    /// Note that this only tells you if it _would_ be empty
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }
}
