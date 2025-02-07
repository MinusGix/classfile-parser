use std::{borrow::Cow, fmt::Display, num::NonZeroUsize};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DescriptorTypeError {
    /// There was no input to parse
    NoInput,
    InvalidTypeOpener,
    /// There was no class name
    EmptyClassName,
    /// There was no ending semicolon for the class name
    NoClassNameEnd,
    /// There were too many arrays nested right after each other such that it exceeded
    /// the levels integer
    TooManyNestedArrays,
}

/// Non-recursive types for descriptor type
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DescriptorTypeBasic<'a> {
    /// B byte
    Byte,
    /// C char
    Char,
    /// D double
    Double,
    /// F float
    Float,
    /// I int
    Int,
    /// J long
    Long,
    /// L*class name here*; reference to the class
    ClassName(Cow<'a, [u8]>),
    /// S short
    Short,
    /// Z boolean
    Boolean,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DescriptorType<'a> {
    Basic(DescriptorTypeBasic<'a>),
    /// [arraytype
    /// This has a level field which allows transformation of arbitrary nesting of arrays
    /// into a single variant and avoiding recursive heap allocation through box
    Array {
        level: NonZeroUsize,
        component: DescriptorTypeBasic<'a>,
    },
}
impl<'a> From<DescriptorTypeBasic<'a>> for DescriptorType<'a> {
    fn from(basic: DescriptorTypeBasic<'a>) -> Self {
        Self::Basic(basic)
    }
}
impl<'a> DescriptorType<'a> {
    pub(crate) fn is_beginning_char(c: u8) -> bool {
        matches!(
            c,
            b'B' | b'C' | b'D' | b'F' | b'I' | b'J' | b'L' | b'S' | b'Z' | b'['
        )
    }

    pub fn to_owned<'b>(self) -> DescriptorType<'b> {
        match self {
            Self::Basic(x) => DescriptorType::Basic(x.to_owned()),
            Self::Array { level, component } => DescriptorType::Array {
                level,
                component: component.to_owned(),
            },
        }
    }

    pub fn parse(
        mut text: &'a [u8],
    ) -> Result<(DescriptorType<'a>, &'a [u8]), DescriptorTypeError> {
        // We use bytes here because this lets us avoid the slightly expensive utf8 iteration
        // Which is thankfully correct since a utf8 character won't have parts that look like ASCII
        // which we use to detect the start and end of parts
        // Though, if this panics, then that means we did it incorrectly.

        let mut iter = text.iter();
        let first = iter.next().ok_or(DescriptorTypeError::NoInput)?;
        let mut latest_index = 1;

        let value: DescriptorType = match first {
            b'B' => DescriptorTypeBasic::Byte.into(),
            b'C' => DescriptorTypeBasic::Char.into(),
            b'D' => DescriptorTypeBasic::Double.into(),
            b'F' => DescriptorTypeBasic::Float.into(),
            b'I' => DescriptorTypeBasic::Int.into(),
            b'J' => DescriptorTypeBasic::Long.into(),
            b'L' => {
                let start_index = latest_index;
                let mut prev_index = start_index;
                let mut found_semicolon = false;
                for c in iter {
                    latest_index += 1;
                    if *c == b';' {
                        found_semicolon = true;
                        break;
                    }
                    prev_index += 1;
                }

                if !found_semicolon {
                    return Err(DescriptorTypeError::NoClassNameEnd);
                }

                // Deliberately not including the closing semicolon
                let class_name = &text[start_index..prev_index];
                if class_name.is_empty() {
                    return Err(DescriptorTypeError::EmptyClassName);
                }

                DescriptorTypeBasic::ClassName(Cow::Borrowed(class_name)).into()
            }
            b'S' => DescriptorTypeBasic::Short.into(),
            b'Z' => DescriptorTypeBasic::Boolean.into(),
            b'[' => {
                // TODO: NonZeroUsize ops would make this a bit nicer
                let mut level = 1usize;
                for c in iter {
                    if *c == b'[' {
                        latest_index += 1;
                        level = level
                            .checked_add(1)
                            .ok_or(DescriptorTypeError::TooManyNestedArrays)?;
                    } else {
                        break;
                    }
                }

                let level = NonZeroUsize::new(level).unwrap();
                let (component, text) = DescriptorType::parse(&text[latest_index..])?;
                let component = match component {
                    DescriptorType::Basic(x) => x,
                    _ => unreachable!(
                        "There should have been no recognizable arrays after arrayp arsing."
                    ),
                };
                // Deliberately returns early
                return Ok((DescriptorType::Array { level, component }, text));
            }
            _ => return Err(DescriptorTypeError::InvalidTypeOpener),
        };

        // If we got here with a latest index of 1 then it has to be a singular ascii character
        text = &text[latest_index..];
        Ok((value, text))
    }
}
impl Display for DescriptorTypeBasic<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DescriptorTypeBasic::Byte => f.write_str("byte"),
            DescriptorTypeBasic::Char => f.write_str("char"),
            DescriptorTypeBasic::Double => f.write_str("double"),
            DescriptorTypeBasic::Float => f.write_str("float"),
            DescriptorTypeBasic::Int => f.write_str("int"),
            DescriptorTypeBasic::Long => f.write_str("long"),
            DescriptorTypeBasic::ClassName(path) => {
                if let Ok(path) = std::str::from_utf8(&path) {
                    f.write_str(path)
                } else {
                    f.write_str("[non-utf8 class name]")
                }
            }
            DescriptorTypeBasic::Short => f.write_str("short"),
            DescriptorTypeBasic::Boolean => f.write_str("boolean"),
        }
    }
}
impl Display for DescriptorType<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Basic(x) => x.fmt(f),
            Self::Array { level, component } => {
                f.write_fmt(format_args!("{}", component))?;
                for _ in 0..level.get() {
                    f.write_str("[]")?;
                }
                Ok(())
            }
        }
    }
}

impl<'a> DescriptorTypeBasic<'a> {
    pub fn to_owned<'b>(self) -> DescriptorTypeBasic<'b> {
        match self {
            DescriptorTypeBasic::ClassName(x) => {
                DescriptorTypeBasic::ClassName(Cow::Owned(x.into_owned()))
            }
            DescriptorTypeBasic::Byte => DescriptorTypeBasic::Byte,
            DescriptorTypeBasic::Char => DescriptorTypeBasic::Char,
            DescriptorTypeBasic::Double => DescriptorTypeBasic::Double,
            DescriptorTypeBasic::Float => DescriptorTypeBasic::Float,
            DescriptorTypeBasic::Int => DescriptorTypeBasic::Int,
            DescriptorTypeBasic::Long => DescriptorTypeBasic::Long,
            DescriptorTypeBasic::Short => DescriptorTypeBasic::Short,
            DescriptorTypeBasic::Boolean => DescriptorTypeBasic::Boolean,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{borrow::Cow, num::NonZeroUsize};

    use crate::descriptor::DescriptorTypeBasic;

    use super::{DescriptorType, DescriptorTypeError};

    #[test]
    fn parsing() -> Result<(), DescriptorTypeError> {
        let one = NonZeroUsize::new(1).unwrap();
        let two = NonZeroUsize::new(2).unwrap();
        assert_eq!(
            DescriptorType::parse(b"B")?,
            (DescriptorTypeBasic::Byte.into(), b"" as &[u8])
        );
        assert_eq!(
            DescriptorType::parse(b"C")?,
            (DescriptorTypeBasic::Char.into(), b"" as &[u8])
        );
        assert_eq!(
            DescriptorType::parse(b"D")?,
            (DescriptorTypeBasic::Double.into(), b"" as &[u8])
        );
        assert_eq!(
            DescriptorType::parse(b"F")?,
            (DescriptorTypeBasic::Float.into(), b"" as &[u8])
        );
        assert_eq!(
            DescriptorType::parse(b"I")?,
            (DescriptorTypeBasic::Int.into(), b"" as &[u8])
        );
        assert_eq!(
            DescriptorType::parse(b"J")?,
            (DescriptorTypeBasic::Long.into(), b"" as &[u8])
        );
        assert_eq!(
            DescriptorType::parse(b"S")?,
            (DescriptorTypeBasic::Short.into(), b"" as &[u8])
        );
        assert_eq!(
            DescriptorType::parse(b"Z")?,
            (DescriptorTypeBasic::Boolean.into(), b"" as &[u8])
        );
        assert_eq!(
            DescriptorType::parse(b"ZB")?,
            (DescriptorTypeBasic::Boolean.into(), b"B" as &[u8])
        );
        assert_eq!(
            DescriptorType::parse(b"ZBLjava/test;")?,
            (
                DescriptorTypeBasic::Boolean.into(),
                b"BLjava/test;" as &[u8]
            )
        );

        // Arrays
        assert_eq!(
            DescriptorType::parse(b"["),
            Err(DescriptorTypeError::NoInput)
        );
        assert_eq!(
            DescriptorType::parse(b"[I")?,
            (
                DescriptorType::Array {
                    level: one,
                    component: DescriptorTypeBasic::Int,
                },
                b"" as &[u8]
            )
        );
        assert_eq!(
            DescriptorType::parse(b"[IB")?,
            (
                DescriptorType::Array {
                    level: one,
                    component: DescriptorTypeBasic::Int,
                },
                b"B" as &[u8]
            )
        );
        assert_eq!(
            DescriptorType::parse(b"[[I")?,
            (
                DescriptorType::Array {
                    level: two,
                    component: DescriptorTypeBasic::Int,
                },
                b"" as &[u8]
            )
        );
        assert_eq!(
            DescriptorType::parse(b"[[IB")?,
            (
                DescriptorType::Array {
                    level: two,
                    component: DescriptorTypeBasic::Int
                },
                b"B" as &[u8]
            )
        );

        // Classes
        assert_eq!(
            DescriptorType::parse(b"L"),
            Err(DescriptorTypeError::NoClassNameEnd)
        );
        assert_eq!(
            DescriptorType::parse(b"L;"),
            Err(DescriptorTypeError::EmptyClassName)
        );
        assert_eq!(
            DescriptorType::parse(b"Ljava/util/Scanner;")?,
            (
                DescriptorTypeBasic::ClassName(Cow::Borrowed(b"java/util/Scanner")).into(),
                b"" as &[u8]
            )
        );

        assert_eq!(
            DescriptorType::parse(b"Ljava/util;B[I")?,
            (
                DescriptorTypeBasic::ClassName(Cow::Borrowed(b"java/util")).into(),
                b"B[I" as &[u8]
            )
        );
        Ok(())
    }
}
