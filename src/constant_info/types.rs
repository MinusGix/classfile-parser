use crate::{impl_from_try_reverse};

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
    pub utf8_string: String,
    pub bytes: Vec<u8>,
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
    pub name_index: u16,
}

#[derive(Clone, Debug)]
pub struct StringConstant {
    pub string_index: u16,
}

#[derive(Clone, Debug)]
pub struct FieldRefConstant {
    pub class_index: u16,
    pub name_and_type_index: u16,
}

#[derive(Clone, Debug)]
pub struct MethodRefConstant {
    pub class_index: u16,
    pub name_and_type_index: u16,
}

#[derive(Clone, Debug)]
pub struct InterfaceMethodRefConstant {
    pub class_index: u16,
    pub name_and_type_index: u16,
}

#[derive(Clone, Debug)]
pub struct NameAndTypeConstant {
    pub name_index: u16,
    pub descriptor_index: u16,
}

#[derive(Clone, Debug)]
pub struct MethodHandleConstant {
    pub reference_kind: u8,
    pub reference_index: u16,
}

#[derive(Clone, Debug)]
pub struct MethodTypeConstant {
    pub descriptor_index: u16,
}

#[derive(Clone, Debug)]
pub struct InvokeDynamicConstant {
    pub bootstrap_method_attr_index: u16,
    pub name_and_type_index: u16,
}
