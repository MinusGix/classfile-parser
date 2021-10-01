use std::ops::{Index, IndexMut};

use attribute_info::AttributeInfo;
use constant_info::ConstantInfo;
use field_info::FieldInfo;
use method_info::MethodInfo;

#[derive(Clone, Debug)]
pub struct ClassFile {
    pub minor_version: u16,
    pub major_version: u16,
    pub const_pool_size: u16,
    pub const_pool: ConstantPool,
    pub access_flags: ClassAccessFlags,
    pub this_class: u16,
    pub super_class: u16,
    pub interfaces_count: u16,
    pub interfaces: Vec<u16>,
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

/// A wrapper structure around Vec to provide access
#[derive(Clone, Debug)]
pub struct ConstantPool {
    pool: Vec<ConstantInfo>,
}
impl ConstantPool {
    // Note: The casts from u16 to usize are always fine if the invariant holds,
    // and different platforms won't have issues since usize is always at least a u16

    /// The constant pool has an invariant that it holds at most u16 elements
    pub(crate) fn new(pool: Vec<ConstantInfo>) -> Self {
        assert!(pool.len() <= (u16::MAX as usize));
        Self {
            pool,
        }
    }

    pub fn len(&self) -> u16 {
        self.pool.len() as u16
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn get(&self, i: u16) -> Option<&ConstantInfo> {
        self.pool.get(i as usize)
    }

    pub fn get_mut(&mut self, i: u16) -> Option<&mut ConstantInfo> {
        self.pool.get_mut(i as usize)
    }

    pub fn iter(&self) -> std::slice::Iter<ConstantInfo> {
        self.pool.iter()
    }
}
impl Index<u16> for ConstantPool {
    type Output = ConstantInfo;
    fn index(&self, index: u16) -> &Self::Output {
        &self.pool[index as usize]
    }
}
impl IndexMut<u16> for ConstantPool {
    fn index_mut(&mut self, index: u16) -> &mut Self::Output {
        &mut self.pool[index as usize]
    }
}