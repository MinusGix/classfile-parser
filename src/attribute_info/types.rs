use crate::{
    constant_info::{ClassConstant, Utf8Constant},
    constant_pool::ConstantPoolIndexRaw,
};

/// An index into the code that should be an index
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct InstructionIndex(pub u16);

#[derive(Clone, Debug)]
pub struct AttributeInfo {
    pub attribute_name_index: ConstantPoolIndexRaw<Utf8Constant>,
    pub attribute_length: u32,
    pub info: Vec<u8>,
}

#[derive(Clone, Debug)]
pub struct ExceptionEntry {
    /// The code range at which the exception handler is active and waiting for an exception
    pub start_pc: InstructionIndex,
    pub end_pc: InstructionIndex,
    /// The location of the exception handler code
    pub handler_pc: InstructionIndex,
    /// The class of exception that it catches.
    /// If it is zero, then it catches all exceptions.
    pub catch_type: ConstantPoolIndexRaw<ClassConstant>,
}

#[derive(Clone, Debug)]
pub struct CodeAttribute {
    pub max_stack: u16,
    pub max_locals: u16,
    pub code_length: u32,
    pub code: Vec<u8>,
    pub exception_table_length: u16,
    pub exception_table: Vec<ExceptionEntry>,
    pub attributes_count: u16,
    pub attributes: Vec<AttributeInfo>,
}

#[derive(Clone, Debug)]
#[repr(u8)]
pub enum VerificationTypeInfo {
    Top = 0,
    Integer = 1,
    Float = 2,
    Double = 3,
    Long = 4,
    Null = 5,
    UninitializedThis = 6,
    Object = 7,
    Uninitialized = 8,
}

#[derive(Clone, Debug)]
pub enum StackMapFrame {
    SameFrame {
        frame_type: u8,
    },
    SameLocals1StackItemFrame {
        frame_type: u8,
        stack: VerificationTypeInfo,
    },
    SameLocals1StackItemFrameExtended {
        frame_type: u8,
        offset_delta: u16,
        stack: VerificationTypeInfo,
    },
    ChopFrame {
        frame_type: u8,
        offset_delta: u16,
    },
    SameFrameExtended {
        frame_type: u8,
        offset_delta: u16,
    },
    AppendFrame {
        frame_type: u8,
        offset_delta: u16,
        locals: Vec<VerificationTypeInfo>,
    },
    FullFrame {
        frame_type: u8,
        offset_delta: u16,
        number_of_locals: u16,
        locals: Vec<VerificationTypeInfo>,
        number_of_stack_items: u16,
        stack: Vec<VerificationTypeInfo>,
    },
}

#[derive(Clone, Debug)]
pub struct StackMapTableAttribute {
    pub number_of_entries: u16,
    pub entries: Vec<StackMapFrame>,
}

#[derive(Clone, Debug)]
pub struct ExceptionsAttribute {
    pub exception_table_length: u16,
    pub exception_table: Vec<ConstantPoolIndexRaw<ClassConstant>>,
}

#[derive(Clone, Debug)]
pub struct ConstantValueAttribute {
    pub constant_value_index: u16,
}

#[derive(Clone, Debug)]
pub struct BootstrapMethod {
    pub bootstrap_method_ref: u16,
    pub num_bootstrap_arguments: u16,
    pub bootstrap_arguments: Vec<u16>,
}

#[derive(Clone, Debug)]
pub struct BootstrapMethodsAttribute {
    pub num_bootstrap_methods: u16,
    pub bootstrap_methods: Vec<BootstrapMethod>,
}

/// The SourceFile attribute is an optional fixed-length attribute in the attributes table of a ClassFile structure (§4.1).
///
/// There may be at most one SourceFile attribute in the attributes table of a ClassFile structure.
/// [see more](https://docs.oracle.com/javase/specs/jvms/se8/html/jvms-4.html#jvms-4.7.10)
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct SourceFileAttribute {
    /// The value of the attribute_name_index item must be a valid index into the constant_pool table.
    /// The constant_pool entry at that index must be a CONSTANT_Utf8_info structure
    /// representing the string "SourceFile".
    pub attribute_name_index: u16,
    /// The value of the attribute_length item must be two.
    pub attribute_length: u32,
    /// The value of the sourcefile_index item must be a valid index into the constant_pool table.
    /// The constant_pool entry at that index must be a CONSTANT_Utf8_info structure representing a string.
    pub sourcefile_index: u16,
}
