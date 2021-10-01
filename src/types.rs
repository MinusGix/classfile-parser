use std::{convert::{TryFrom, TryInto}, marker::PhantomData};

use attribute_info::AttributeInfo;
use constant_info::ConstantInfo;
use field_info::FieldInfo;
use method_info::MethodInfo;

use crate::constant_info::ClassConstant;

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

/// An index into the constant pool that hasn't been offset by -1
#[derive(Debug, PartialEq, Eq)]
pub struct ConstantPoolIndexRaw<T>(pub u16, PhantomData<*const T>);
impl<T> ConstantPoolIndexRaw<T> {
    pub fn new(i: u16) -> Self {
        Self(i, PhantomData)
    }
}
impl<T> Clone for ConstantPoolIndexRaw<T> {
    fn clone(&self) -> Self {
        Self::new(self.0)
    }
}
impl<T> Copy for ConstantPoolIndexRaw<T> {}
impl<T: TryFrom<ConstantInfo>> ConstantPoolIndexRaw<T> {
    pub fn into_generic(self) -> ConstantPoolIndexRaw<ConstantInfo> {
        ConstantPoolIndexRaw(self.0, PhantomData)
    }
}

/// A constant pool index that has already been offset by -1
#[derive(Debug, PartialEq, Eq)]
pub struct ConstantPoolIndex<T>(pub u16, PhantomData<*const T>);
impl<T> ConstantPoolIndex<T> {
    pub fn new(i: u16) -> Self {
        Self(i, PhantomData)
    }
}
impl<T> Clone for ConstantPoolIndex<T> {
    fn clone(&self) -> Self {
        Self::new(self.0)
    }
}
impl<T> Copy for ConstantPoolIndex<T> {}
impl<T: TryFrom<ConstantInfo>> ConstantPoolIndex<T> {
    pub fn into_generic(self) -> ConstantPoolIndex<ConstantInfo> {
        ConstantPoolIndex(self.0, PhantomData)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct InvalidConstantPoolIndex;
// we use TryFrom because the raw index could be 0, and we can't represent -1
impl<T: TryFrom<ConstantInfo>> TryFrom<ConstantPoolIndexRaw<T>> for ConstantPoolIndex<T> {
    type Error = InvalidConstantPoolIndex;

    fn try_from(value: ConstantPoolIndexRaw<T>) -> Result<Self, Self::Error> {
        value.0
            .checked_sub(1)
            .map(ConstantPoolIndex::<T>::new)
            .ok_or(InvalidConstantPoolIndex)
    }
}
impl TryFrom<u16> for ConstantPoolIndex<ConstantInfo> {
    type Error = ();
    fn try_from(value: u16) -> Result<Self, Self::Error> {
        Ok(ConstantPoolIndex::new(value))
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

    pub fn get<T>(&self, i: impl TryInto<ConstantPoolIndex<T>>) -> Option<&ConstantInfo> {
        let i: ConstantPoolIndex<T> = i.try_into().ok()?;
        self.pool.get(i.0 as usize)
    }

    pub fn get_mut<T>(&mut self, i: impl TryInto<ConstantPoolIndex<T>>) -> Option<&mut ConstantInfo> {
        let i: ConstantPoolIndex<T> = i.try_into().ok()?;
        self.pool.get_mut(i.0 as usize)
    }

    pub fn get_t<'a, T>(&'a self, i: impl TryInto<ConstantPoolIndex<T>>) -> Option<&'a T> where &'a T: TryFrom<&'a ConstantInfo> {
        let i: ConstantPoolIndex<T> = i.try_into().ok()?;
        let v: &'a ConstantInfo = self.pool.get(i.0 as usize)?;
        <&'a T>::try_from(v).ok()
    }

    pub fn get_t_mut<'a, T>(&'a mut self, i: impl TryInto<ConstantPoolIndex<T>>) -> Option<&'a mut T> where &'a mut T: TryFrom<&'a mut ConstantInfo> {
        let i: ConstantPoolIndex<T> = i.try_into().ok()?;
        let v: &'a mut ConstantInfo = self.pool.get_mut(i.0 as usize)?;
        <&'a mut T>::try_from(v).ok()
    }

    pub fn iter(&self) -> std::slice::Iter<ConstantInfo> {
        self.pool.iter()
    }
}

// TODO: Implementing Index{Mut,} would be useful, but I failed to make it work properly