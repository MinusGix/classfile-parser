use std::fmt::Display;


#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DescriptorTypeError {
    /// There was no input to parse
    NoInput,
    InvalidTypeOpener,
    EmptyClassName,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DescriptorType {
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
    ClassName(Vec<String>),
    /// S short
    Short,
    /// Z boolean
    Boolean,
    // TODO: We could have a variable for how many nested arrays it is?
    /// Z boolean
    Array(Box<DescriptorType>),
}
impl DescriptorType {
    pub(crate) fn is_beginning_char(c: char) -> bool {
        matches!(c, 'B' | 'C' | 'D' | 'F' | 'I' | 'J' | 'L' | 'S' | 'Z' | '[')
    }

    pub fn parse(mut text: &str) -> Result<(DescriptorType, &str), DescriptorTypeError> {
        let mut iter = text.char_indices().peekable();
        let (latest_index, first) = iter.next().ok_or(DescriptorTypeError::NoInput)?;
        let mut latest_index = latest_index + first.len_utf8();

        let value = match first {
            'B' => Self::Byte,
            'C' => Self::Char,
            'D' => Self::Double,
            'F' => Self::Float,
            'I' => Self::Int,
            'J' => Self::Long,
            'L' => {
                let mut path = Vec::new();
                for (i, c) in iter {
                    if c == ';' {
                        path.push((&text[latest_index..i]).to_owned());
                        latest_index = i + c.len_utf8();
                        break;
                    } else if c == '/' {
                        path.push((&text[latest_index..i]).to_owned());
                        latest_index = i + c.len_utf8();
                    } else {
                        // ignore, it will automatically get pushed
                    }
                }

                if path.is_empty() || path.first().map(String::is_empty).unwrap_or(false) {
                    return Err(DescriptorTypeError::EmptyClassName);
                }

                Self::ClassName(path)
            },
            'S' => Self::Short,
            'Z' => Self::Boolean,
            '[' => {
                let (typ, text) = DescriptorType::parse(&text[latest_index..])?;
                return Ok((Self::Array(Box::new(typ)), text))
            },
            _ => return Err(DescriptorTypeError::InvalidTypeOpener),
        };

        text = &text[latest_index..];
        Ok((value, text))
    }
}
impl Display for DescriptorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Byte => f.write_str("byte"),
            Self::Char => f.write_str("char"),
            Self::Double => f.write_str("double"),
            Self::Float => f.write_str("float"),
            Self::Int => f.write_str("int"),
            Self::Long => f.write_str("long"),
            Self::ClassName(path) => f.write_str(&path.join(".")),
            Self::Short => f.write_str("short"),
            Self::Boolean => f.write_str("boolean"),
            Self::Array(typ) => f.write_fmt(format_args!("{}[]", typ.as_ref()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{DescriptorType, DescriptorTypeError};

    #[test]
    fn parsing() -> Result<(), DescriptorTypeError> {
        assert_eq!(DescriptorType::parse("B")?, (DescriptorType::Byte, ""));
        assert_eq!(DescriptorType::parse("C")?, (DescriptorType::Char, ""));
        assert_eq!(DescriptorType::parse("D")?, (DescriptorType::Double, ""));
        assert_eq!(DescriptorType::parse("F")?, (DescriptorType::Float, ""));
        assert_eq!(DescriptorType::parse("I")?, (DescriptorType::Int, ""));
        assert_eq!(DescriptorType::parse("J")?, (DescriptorType::Long, ""));
        assert_eq!(DescriptorType::parse("S")?, (DescriptorType::Short, ""));
        assert_eq!(DescriptorType::parse("Z")?, (DescriptorType::Boolean, ""));
        assert_eq!(DescriptorType::parse("ZB")?, (DescriptorType::Boolean, "B"));
        assert_eq!(DescriptorType::parse("ZBLjava/test;")?, (DescriptorType::Boolean, "BLjava/test;"));
        
        // Arrays
        assert_eq!(DescriptorType::parse("["), Err(DescriptorTypeError::NoInput));
        assert_eq!(DescriptorType::parse("[I")?, (DescriptorType::Array(Box::new(DescriptorType::Int)), ""));
        assert_eq!(DescriptorType::parse("[IB")?, (DescriptorType::Array(Box::new(DescriptorType::Int)), "B"));
        assert_eq!(DescriptorType::parse("[[I")?, (DescriptorType::Array(Box::new(DescriptorType::Array(Box::new(DescriptorType::Int)))), ""));
        assert_eq!(DescriptorType::parse("[[IB")?, (DescriptorType::Array(Box::new(DescriptorType::Array(Box::new(DescriptorType::Int)))), "B"));
        
        // Classes
        assert_eq!(DescriptorType::parse("L"), Err(DescriptorTypeError::EmptyClassName));
        assert_eq!(DescriptorType::parse("L;"), Err(DescriptorTypeError::EmptyClassName));
        assert_eq!(DescriptorType::parse("Ljava/util/Scanner;")?, (DescriptorType::ClassName(vec!
            ["java".to_owned(), "util".to_owned(), "Scanner".to_owned()]), ""));

        assert_eq!(DescriptorType::parse("Ljava/util;B[I")?, (DescriptorType::ClassName(vec!["java".to_owned(), "util".to_owned()]), "B[I"));
        Ok(())
    }
}
