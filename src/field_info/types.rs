use smallvec::SmallVec;

use crate::attribute_info::AttributeInfo;

use crate::{constant_info::Utf8Constant, constant_pool::ConstantPoolIndexRaw};

#[derive(Clone, Debug)]
pub struct FieldInfo {
    pub access_flags: FieldAccessFlags,
    pub name_index: ConstantPoolIndexRaw<Utf8Constant>,
    pub descriptor_index: ConstantPoolIndexRaw<Utf8Constant>,
    pub attributes_count: u16,
    pub attributes: SmallVec<[AttributeInfo; 2]>,
}

#[derive(Clone, Debug)]
pub struct FieldInfoOpt {
    pub access_flags: FieldAccessFlags,
    pub name_index: ConstantPoolIndexRaw<Utf8Constant>,
    pub descriptor_index: ConstantPoolIndexRaw<Utf8Constant>,
    pub attributes_count: u16
}

bitflags! {
    pub struct FieldAccessFlags: u16 {
        const PUBLIC = 0x0001;     // 	Declared public; may be accessed from outside its package.
        const PRIVATE = 0x0002;    // 	Declared private; usable only within the defining class.
        const PROTECTED = 0x0004;  // 	Declared protected; may be accessed within subclasses.
        const STATIC = 0x0008;     // 	Declared static.
        const FINAL = 0x0010;      // 	Declared final; never directly assigned to after object construction.
        const VOLATILE = 0x0040;   // 	Declared volatile; cannot be cached.
        const TRANSIENT = 0x0080;  // 	Declared transient; not written or read by a persistent object manager.
        const SYNTHETIC = 0x1000;  // 	Declared synthetic; not present in the source code.
        const ANNOTATION = 0x2000; // 	Declared as an annotation type.
        const ENUM = 0x4000;       // 	Declared as an element of an enum.
    }
}
