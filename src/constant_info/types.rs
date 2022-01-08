use std::borrow::Cow;

use crate::{constant_pool::ConstantPoolIndexRaw, impl_from_try_reverse};

#[derive(Clone, Debug)]
pub enum ConstantInfo {
    Utf8(Utf8Constant),
    Integer(IntegerConstant),
    Float(FloatConstant),
    Long(LongConstant),
    Double(DoubleConstant),
    Class(ClassConstant),
    String(StringConstant),
    FieldRef(FieldRefConstant),
    MethodRef(MethodRefConstant),
    InterfaceMethodRef(InterfaceMethodRefConstant),
    NameAndType(NameAndTypeConstant),
    MethodHandle(MethodHandleConstant),
    MethodType(MethodTypeConstant),
    InvokeDynamic(InvokeDynamicConstant),
    /// The unusuable variant appears right after the Double/Long types
    /// This is technically not in the actual file, but it represents the latter
    /// 4 bytes of the variant. It still has its own index, and so it is represented
    /// as an unusable variant here.
    Unusable,
}

/// The constant was not of the correct type
#[derive(Debug, Clone)]
pub struct IncorrectConstant;

impl_from_try_reverse!(enum Utf8Constant => ConstantInfo::Utf8; IncorrectConstant);
impl_from_try_reverse!(enum IntegerConstant => ConstantInfo::Integer; IncorrectConstant);
impl_from_try_reverse!(enum FloatConstant => ConstantInfo::Float; IncorrectConstant);
impl_from_try_reverse!(enum LongConstant => ConstantInfo::Long; IncorrectConstant);
impl_from_try_reverse!(enum DoubleConstant => ConstantInfo::Double; IncorrectConstant);
impl_from_try_reverse!(enum ClassConstant => ConstantInfo::Class; IncorrectConstant);
impl_from_try_reverse!(enum StringConstant => ConstantInfo::String; IncorrectConstant);
impl_from_try_reverse!(enum FieldRefConstant => ConstantInfo::FieldRef; IncorrectConstant);
impl_from_try_reverse!(enum MethodRefConstant => ConstantInfo::MethodRef; IncorrectConstant);
impl_from_try_reverse!(enum InterfaceMethodRefConstant => ConstantInfo::InterfaceMethodRef; IncorrectConstant);
impl_from_try_reverse!(enum NameAndTypeConstant => ConstantInfo::NameAndType; IncorrectConstant);
impl_from_try_reverse!(enum MethodHandleConstant => ConstantInfo::MethodHandle; IncorrectConstant);
impl_from_try_reverse!(enum MethodTypeConstant => ConstantInfo::MethodType; IncorrectConstant);
impl_from_try_reverse!(enum InvokeDynamicConstant => ConstantInfo::InvokeDynamic; IncorrectConstant);
// TODO: From Unusuable?

#[derive(Clone, Debug)]
pub struct Utf8Constant {
    pub bytes: Vec<u8>,
    text: Option<String>,
}
impl Utf8Constant {
    pub(crate) fn new(bytes: Vec<u8>) -> Utf8Constant {
        Utf8Constant { bytes, text: None }
    }

    /// Convert this to text
    /// Note that this at times has to do an allocation on first call, but is cached for later calls
    pub fn as_text_mut(&mut self) -> &str {
        // This weird check and unwrap is to avoid the borrow checker
        if self.text.is_some() {
            self.text.as_ref().unwrap().as_str()
        } else {
            let text = cesu8::from_java_cesu8(&self.bytes)
                .unwrap_or_else(|_| String::from_utf8_lossy(&self.bytes));
            match text {
                Cow::Borrowed(text) => text,
                Cow::Owned(text) => {
                    self.text = Some(text);
                    let text = self.text.as_ref().unwrap().as_str();
                    text
                }
            }
        }
    }

    /// Convert this to text.
    /// Note that this at times has to do an allocation on first call.
    /// You should prefer using `as_text_mut` since that caches.
    pub fn as_text(&self) -> Cow<str> {
        if let Some(text) = &self.text {
            Cow::Borrowed(text.as_str())
        } else {
            let text = cesu8::from_java_cesu8(&self.bytes)
                .unwrap_or_else(|_| String::from_utf8_lossy(&self.bytes));
            // We can't store the owned version since we don't have mutable access to cache it
            text
        }
    }
}

#[derive(Clone, Debug)]
pub struct IntegerConstant {
    pub value: i32,
}

#[derive(Clone, Debug)]
pub struct FloatConstant {
    pub value: f32,
}

#[derive(Clone, Debug)]
pub struct LongConstant {
    pub value: i64,
}

#[derive(Clone, Debug)]
pub struct DoubleConstant {
    pub value: f64,
}

#[derive(Clone, Debug)]
pub struct ClassConstant {
    pub name_index: ConstantPoolIndexRaw<Utf8Constant>,
}

#[derive(Clone, Debug)]
pub struct StringConstant {
    pub string_index: ConstantPoolIndexRaw<Utf8Constant>,
}

#[derive(Clone, Debug)]
pub struct FieldRefConstant {
    /// Must be class or interface
    pub class_index: ConstantPoolIndexRaw<ClassConstant>,
    /// Must be a field or method descriptor
    pub name_and_type_index: ConstantPoolIndexRaw<NameAndTypeConstant>,
}

#[derive(Clone, Debug)]
pub struct MethodRefConstant {
    /// Must be class
    pub class_index: ConstantPoolIndexRaw<ClassConstant>,
    /// Must be for <init> method ref
    pub name_and_type_index: ConstantPoolIndexRaw<NameAndTypeConstant>,
}

#[derive(Clone, Debug)]
pub struct InterfaceMethodRefConstant {
    /// Must be interface
    pub class_index: ConstantPoolIndexRaw<ClassConstant>,
    pub name_and_type_index: ConstantPoolIndexRaw<NameAndTypeConstant>,
}

#[derive(Clone, Debug)]
pub struct NameAndTypeConstant {
    pub name_index: ConstantPoolIndexRaw<Utf8Constant>,
    pub descriptor_index: ConstantPoolIndexRaw<Utf8Constant>,
}

#[derive(Clone, Debug)]
pub struct MethodHandleConstant {
    pub reference_kind: u8,
    // We don't know the exact type for this, since it depends upon reference kind
    pub reference_index: ConstantPoolIndexRaw<ConstantInfo>,
}

#[derive(Clone, Debug)]
pub struct MethodTypeConstant {
    pub descriptor_index: ConstantPoolIndexRaw<Utf8Constant>,
}

#[derive(Clone, Debug)]
pub struct InvokeDynamicConstant {
    pub bootstrap_method_attr_index: u16,
    pub name_and_type_index: ConstantPoolIndexRaw<NameAndTypeConstant>,
}
