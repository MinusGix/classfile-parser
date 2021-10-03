use super::types::{DescriptorType, DescriptorTypeError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MethodDescriptorError<'a> {
    Empty,
    NoOpeningBracket,
    NoClosingBracket,
    /// The error and the index of the parameter
    ParameterTypeError(DescriptorTypeError, usize),
    ReturnTypeError(DescriptorTypeError),
    NoReturnType,
    RemainingData(&'a str),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MethodDescriptor {
    pub parameter_types: Vec<DescriptorType>,
    /// If this is None, then the return type was void
    pub return_type: Option<DescriptorType>,
}
impl MethodDescriptor {
    // TODO: Settings that allow the parsing to be more permissive?
    /// Note: We currently don't uphold the JVM restriction of the method descriptor being at most
    /// 255 bytes.
    pub fn parse(mut text: &str) -> Result<MethodDescriptor, MethodDescriptorError<'_>> {
        if text.is_empty() {
            return Err(MethodDescriptorError::Empty);
        }

        if !text.starts_with('(') {
            return Err(MethodDescriptorError::NoOpeningBracket);
        }

        text = &text[1..];

        let mut parameter_types = Vec::new();

        while text.chars().next().map(DescriptorType::is_beginning_char).unwrap_or(false) {
            let (parameter, after_text) = DescriptorType::parse(text)
                .map_err(|x| MethodDescriptorError::ParameterTypeError(x, parameter_types.len()))?;
            text = after_text;
            parameter_types.push(parameter);
        }

        if !text.starts_with(')') {
            return Err(MethodDescriptorError::NoClosingBracket);
        }

        text = &text[1..];

        let ch = text.chars().next();
        let return_type = if let Some(ch) = ch {
            if ch == 'V' {
                None
            } else {
                // Otherwise try parsing it as a type, and just use that error
                let (typ, after_text) = DescriptorType::parse(text)
                    .map_err(MethodDescriptorError::ReturnTypeError)?;
                if !after_text.is_empty() {
                    // There was remaining unhandled data, which means we parsed this incorrectly somehow
                    return Err(MethodDescriptorError::RemainingData(after_text));
                }
                Some(typ)
            }
        } else {
            return Err(MethodDescriptorError::NoReturnType);
        };

        Ok(MethodDescriptor {
            parameter_types,
            return_type,
        })
    }
}
impl std::fmt::Display for MethodDescriptor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("(")?;
        for (i, param) in self.parameter_types.iter().enumerate() {
            f.write_fmt(format_args!("{}", param))?;
            if (i + 1) != self.parameter_types.len() {
                f.write_str(", ")?;
            }
        }
        f.write_str(")")?;

        if let Some(ret) = &self.return_type {
            f.write_fmt(format_args!(": {}", ret))?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::descriptor::{method::{MethodDescriptor, MethodDescriptorError}, types::{DescriptorType, DescriptorTypeError}};

    #[test]
    fn parsing() {
        assert_eq!(MethodDescriptor::parse(""), Err(MethodDescriptorError::Empty));
        assert_eq!(MethodDescriptor::parse(")"), Err(MethodDescriptorError::NoOpeningBracket));
        assert_eq!(MethodDescriptor::parse("("), Err(MethodDescriptorError::NoClosingBracket));
        assert_eq!(MethodDescriptor::parse("()"), Err(MethodDescriptorError::NoReturnType));
        assert_eq!(MethodDescriptor::parse("()R"), Err(MethodDescriptorError::ReturnTypeError(DescriptorTypeError::InvalidTypeOpener)));
        assert_eq!(MethodDescriptor::parse("()V"), Ok(MethodDescriptor {
            parameter_types: Vec::new(),
            return_type: None,
        }));
        assert_eq!(MethodDescriptor::parse("(I)V"), Ok(MethodDescriptor {
            parameter_types: vec![DescriptorType::Int],
            return_type: None,
        }));
        assert_eq!(MethodDescriptor::parse("(IDJ)V"), Ok(MethodDescriptor {
            parameter_types: vec![DescriptorType::Int, DescriptorType::Double, DescriptorType::Long],
            return_type: None,
        }));
        assert_eq!(MethodDescriptor::parse("(IDLjava/lang/Thread;)Ljava/lang/Object;"), Ok(MethodDescriptor {
            parameter_types: vec![
                DescriptorType::Int,
                DescriptorType::Double,
                DescriptorType::ClassName(
                    vec!["java".to_owned(), "lang".to_owned(), "Thread".to_owned()]
                )
            ],
            return_type: Some(DescriptorType::ClassName(vec!["java".to_owned(), "lang".to_owned(), "Object".to_owned()])),
        }));
    }
}