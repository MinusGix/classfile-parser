use std::{convert::{TryFrom, TryInto}, ops::{Index, IndexMut}};

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
    pub interfaces: Vec<ConstantPoolIndexRaw>,
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

/// An index into the constant pool that hasn't been offset by -1
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConstantPoolIndexRaw(pub u16);

/// A constant pool index that has already been offset by -1
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConstantPoolIndex(pub u16);

#[derive(Debug, Clone, Copy)]
pub struct InvalidConstantPoolIndex;
// we use TryFrom because the raw index could be 0, and we can't represent -1
impl TryFrom<ConstantPoolIndexRaw> for ConstantPoolIndex {
    type Error = InvalidConstantPoolIndex;

    fn try_from(value: ConstantPoolIndexRaw) -> Result<Self, Self::Error> {
        value.0
            .checked_sub(1)
            .map(ConstantPoolIndex)
            .ok_or(InvalidConstantPoolIndex)
    }
}
impl TryFrom<u16> for ConstantPoolIndex {
    type Error = ();
    fn try_from(value: u16) -> Result<Self, Self::Error> {
        Ok(ConstantPoolIndex(value))
    }
}

/// A wrapper structure around Vec to provide access
#[derive(Clone, Debug)]
pub struct ConstantPool {
    /// In the jvm, the constant pool starts at 1, so the indices start at one.
    /// But this is indexed starting at zero.
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

    pub fn get(&self, i: impl TryInto<ConstantPoolIndex>) -> Option<&ConstantInfo> {
        let i: ConstantPoolIndex = i.try_into().ok()?;
        self.pool.get(i.0 as usize)
    }

    pub fn get_mut(&mut self, i: impl TryInto<ConstantPoolIndex>) -> Option<&mut ConstantInfo> {
        let i: ConstantPoolIndex = i.try_into().ok()?;
        self.pool.get_mut(i.0 as usize)
    }

    pub fn iter(&self) -> std::slice::Iter<ConstantInfo> {
        self.pool.iter()
    }
}
impl<I: TryInto<ConstantPoolIndex>> Index<I> for ConstantPool 
    where <I as TryInto<ConstantPoolIndex>>::Error: std::fmt::Debug {
    type Output = ConstantInfo;
    fn index(&self, index: I) -> &Self::Output {
        let index: ConstantPoolIndex = index.try_into().expect("Invalid index into constant pool");
        &self.pool[index.0 as usize]
    }
}
impl<I: TryInto<ConstantPoolIndex>> IndexMut<I> for ConstantPool 
    where <I as TryInto<ConstantPoolIndex>>::Error: std::fmt::Debug {
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        let index: ConstantPoolIndex = index.try_into().expect("Invalid index into constant pool");
        &mut self.pool[index.0 as usize]
    }
}