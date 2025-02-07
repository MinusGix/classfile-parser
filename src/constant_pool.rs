use std::{
    convert::{TryFrom, TryInto},
    hash::Hash,
    marker::PhantomData,
    rc::Rc,
};

use crate::constant_info::ConstantInfo;

/// An index into the constant pool that hasn't been offset by -1
#[derive(Debug)]
pub struct ConstantPoolIndexRaw<T>(pub u16, PhantomData<*const T>);
impl<T> ConstantPoolIndexRaw<T> {
    pub fn new(i: u16) -> Self {
        Self(i, PhantomData)
    }

    pub fn is_zero(&self) -> bool {
        self.0 == 0
    }
}
impl<T> Eq for ConstantPoolIndexRaw<T> {}
impl<T> PartialEq for ConstantPoolIndexRaw<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}
impl<T> Hash for ConstantPoolIndexRaw<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state)
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
#[derive(Debug)]
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
impl<T> Eq for ConstantPoolIndex<T> {}
impl<T> PartialEq for ConstantPoolIndex<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}
impl<T> Hash for ConstantPoolIndex<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct InvalidConstantPoolIndex;
// we use TryFrom because the raw index could be 0, and we can't represent -1
impl<T: TryFrom<ConstantInfo>> TryFrom<ConstantPoolIndexRaw<T>> for ConstantPoolIndex<T> {
    type Error = InvalidConstantPoolIndex;

    fn try_from(value: ConstantPoolIndexRaw<T>) -> Result<Self, Self::Error> {
        value
            .0
            .checked_sub(1)
            .map(ConstantPoolIndex::<T>::new)
            .ok_or(InvalidConstantPoolIndex)
    }
}
impl<'a, T: TryFrom<ConstantInfo>> TryFrom<&'a ConstantPoolIndexRaw<T>> for ConstantPoolIndex<T> {
    type Error = InvalidConstantPoolIndex;

    fn try_from(value: &'a ConstantPoolIndexRaw<T>) -> Result<Self, Self::Error> {
        let value = *value;
        ConstantPoolIndex::<T>::try_from(value)
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
    pool: Rc<[ConstantInfo]>,
}
impl ConstantPool {
    // Note: The casts from u16 to usize are always fine if the invariant holds,
    // and different platforms won't have issues since usize is always at least a u16

    /// The constant pool has an invariant that it holds at most u16 elements
    pub(crate) fn new(pool: Vec<ConstantInfo>) -> Self {
        assert!(pool.len() <= (u16::MAX as usize));
        Self {
            pool: Rc::from(pool),
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

    pub fn get_t<'a, T>(&'a self, i: impl TryInto<ConstantPoolIndex<T>>) -> Option<&'a T>
    where
        &'a T: TryFrom<&'a ConstantInfo>,
    {
        let i: ConstantPoolIndex<T> = i.try_into().ok()?;
        let v: &'a ConstantInfo = self.pool.get(i.0 as usize)?;
        <&'a T>::try_from(v).ok()
    }

    pub fn iter(&self) -> std::slice::Iter<ConstantInfo> {
        self.pool.iter()
    }
}
/// This is primarily for swapping it out
impl Default for ConstantPool {
    fn default() -> Self {
        Self::new(Vec::new())
    }
}

// TODO: Implementing Index{Mut,} would be useful, but I failed to make it work properly
