use crate::attribute_info::AttributeInfo;
use crate::field_info::FieldInfo;
use crate::method_info::MethodInfo;

use crate::{constant_info::ClassConstant, constant_pool::{ConstantPool, ConstantPoolIndexRaw}};

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

#[derive(Debug, Clone, Copy)]
pub struct ClassFileVersion {
    pub major: u16,
    pub minor: u16,
}
impl ClassFileVersion{
    pub fn into_java_version(self) -> Option<ClassFileJavaVersion> {
        ClassFileJavaVersion::from_version(self.major, self.minor)
    }
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
    pub interfaces: Vec<ConstantPoolIndexRaw<ClassConstant>>,
    pub fields_count: u16,
    pub fields: Vec<FieldInfo>,
    pub methods_count: u16,
    pub methods: Vec<MethodInfo>,
    pub attributes_count: u16,
    pub attributes: Vec<AttributeInfo>,
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