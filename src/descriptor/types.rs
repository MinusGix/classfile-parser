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
    ClassName(Cow<'a, str>),
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
    pub(crate) fn is_beginning_char(c: char) -> bool {
        matches!(c, 'B' | 'C' | 'D' | 'F' | 'I' | 'J' | 'L' | 'S' | 'Z' | '[')
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

    pub fn parse(mut text: &'a str) -> Result<(DescriptorType<'a>, &'a str), DescriptorTypeError> {
        let mut iter = text.char_indices().peekable();
        let (latest_index, first) = iter.next().ok_or(DescriptorTypeError::NoInput)?;
        let mut latest_index = latest_index + first.len_utf8();

        let value: DescriptorType = match first {
            'B' => DescriptorTypeBasic::Byte.into(),
            'C' => DescriptorTypeBasic::Char.into(),
            'D' => DescriptorTypeBasic::Double.into(),
            'F' => DescriptorTypeBasic::Float.into(),
            'I' => DescriptorTypeBasic::Int.into(),
            'J' => DescriptorTypeBasic::Long.into(),
            'L' => {
                let start_index = latest_index;
                let mut prev_index = start_index;
                let mut found_semicolon = false;
                for (i, c) in iter {
                    latest_index = i + c.len_utf8();
                    if c == ';' {
                        found_semicolon = true;
                        break;
                    }
                    prev_index = i + c.len_utf8();
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
            'S' => DescriptorTypeBasic::Short.into(),
            'Z' => DescriptorTypeBasic::Boolean.into(),
            '[' => {
                // TODO: NonZeroUsize ops would make this a bit nicer
                let mut level = 1usize;
                for (i, c) in iter {
                    if c == '[' {
                        latest_index = i + c.len_utf8();
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
            DescriptorTypeBasic::ClassName(path) => f.write_str(path.as_ref()),
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
                DescriptorTypeBasic::ClassName(Cow::Owned(x.to_string()))
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
    use std::num::NonZeroUsize;

    use crate::descriptor::DescriptorTypeBasic;

    use super::{DescriptorType, DescriptorTypeError};

    #[test]
    fn parsing() -> Result<(), DescriptorTypeError> {
        let one = NonZeroUsize::new(1).unwrap();
        let two = NonZeroUsize::new(2).unwrap();
        assert_eq!(
            DescriptorType::parse("B")?,
            (DescriptorTypeBasic::Byte.into(), "")
        );
        assert_eq!(
            DescriptorType::parse("C")?,
            (DescriptorTypeBasic::Char.into(), "")
        );
        assert_eq!(
            DescriptorType::parse("D")?,
            (DescriptorTypeBasic::Double.into(), "")
        );
        assert_eq!(
            DescriptorType::parse("F")?,
            (DescriptorTypeBasic::Float.into(), "")
        );
        assert_eq!(
            DescriptorType::parse("I")?,
            (DescriptorTypeBasic::Int.into(), "")
        );
        assert_eq!(
            DescriptorType::parse("J")?,
            (DescriptorTypeBasic::Long.into(), "")
        );
        assert_eq!(
            DescriptorType::parse("S")?,
            (DescriptorTypeBasic::Short.into(), "")
        );
        assert_eq!(
            DescriptorType::parse("Z")?,
            (DescriptorTypeBasic::Boolean.into(), "")
        );
        assert_eq!(
            DescriptorType::parse("ZB")?,
            (DescriptorTypeBasic::Boolean.into(), "B")
        );
        assert_eq!(
            DescriptorType::parse("ZBLjava/test;")?,
            (DescriptorTypeBasic::Boolean.into(), "BLjava/test;")
        );

        // Arrays
        assert_eq!(
            DescriptorType::parse("["),
            Err(DescriptorTypeError::NoInput)
        );
        assert_eq!(
            DescriptorType::parse("[I")?,
            (
                DescriptorType::Array {
                    level: one,
                    component: DescriptorTypeBasic::Int,
                },
                ""
            )
        );
        assert_eq!(
            DescriptorType::parse("[IB")?,
            (
                DescriptorType::Array {
                    level: one,
                    component: DescriptorTypeBasic::Int,
                },
                "B"
            )
        );
        assert_eq!(
            DescriptorType::parse("[[I")?,
            (
                DescriptorType::Array {
                    level: two,
                    component: DescriptorTypeBasic::Int,
                },
                ""
            )
        );
        assert_eq!(
            DescriptorType::parse("[[IB")?,
            (
                DescriptorType::Array {
                    level: two,
                    component: DescriptorTypeBasic::Int
                },
                "B"
            )
        );

        // Classes
        assert_eq!(
            DescriptorType::parse("L"),
            Err(DescriptorTypeError::NoClassNameEnd)
        );
        assert_eq!(
            DescriptorType::parse("L;"),
            Err(DescriptorTypeError::EmptyClassName)
        );
        assert_eq!(
            DescriptorType::parse("Ljava/util/Scanner;")?,
            (
                DescriptorTypeBasic::ClassName("java/util/Scanner".into()).into(),
                ""
            )
        );

        assert_eq!(
            DescriptorType::parse("Ljava/util;B[I")?,
            (
                DescriptorTypeBasic::ClassName("java/util".into()).into(),
                "B[I"
            )
        );
        Ok(())
    }
}
