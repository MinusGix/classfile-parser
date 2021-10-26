use super::types::{DescriptorType, DescriptorTypeError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MethodDescriptorError {
    Empty,
    NoOpeningBracket,
    NoClosingBracket,
    /// The error and the index of the parameter
    ParameterTypeError(DescriptorTypeError, usize),
    ReturnTypeError(DescriptorTypeError),
    NoReturnType,
    RemainingData,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MethodDescriptor<'a> {
    pub parameter_types: Vec<DescriptorType<'a>>,
    /// If this is None, then the return type was void
    pub return_type: Option<DescriptorType<'a>>,
}
impl<'a> MethodDescriptor<'a> {
    // TODO: Settings that allow the parsing to be more permissive?
    /// Note: We currently don't uphold the JVM restriction of the method descriptor being at most
    /// 255 bytes.
    pub fn parse(mut text: &'a str) -> Result<MethodDescriptor<'a>, MethodDescriptorError> {
        if text.is_empty() {
            return Err(MethodDescriptorError::Empty);
        }

        if !text.starts_with('(') {
            return Err(MethodDescriptorError::NoOpeningBracket);
        }

        text = &text[1..];

        let mut parameter_types = Vec::new();

        while text
            .chars()
            .next()
            .map(DescriptorType::is_beginning_char)
            .unwrap_or(false)
        {
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
                let (typ, after_text) =
                    DescriptorType::parse(text).map_err(MethodDescriptorError::ReturnTypeError)?;
                if !after_text.is_empty() {
                    // There was remaining unhandled data, which means we parsed this incorrectly somehow
                    return Err(MethodDescriptorError::RemainingData);
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

    pub fn to_owned<'b>(self) -> MethodDescriptor<'b> {
        MethodDescriptor {
            parameter_types: self
                .parameter_types
                .into_iter()
                .map(|x| x.to_owned())
                .collect(),
            return_type: self.return_type.map(|x| x.to_owned()),
        }
    }
}
impl std::fmt::Display for MethodDescriptor<'_> {
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
    use crate::descriptor::{
        method::{MethodDescriptor, MethodDescriptorError},
        types::{DescriptorTypeBasic, DescriptorTypeError},
    };

    #[test]
    fn parsing() {
        assert_eq!(
            MethodDescriptor::parse(""),
            Err(MethodDescriptorError::Empty)
        );
        assert_eq!(
            MethodDescriptor::parse(")"),
            Err(MethodDescriptorError::NoOpeningBracket)
        );
        assert_eq!(
            MethodDescriptor::parse("("),
            Err(MethodDescriptorError::NoClosingBracket)
        );
        assert_eq!(
            MethodDescriptor::parse("()"),
            Err(MethodDescriptorError::NoReturnType)
        );
        assert_eq!(
            MethodDescriptor::parse("()R"),
            Err(MethodDescriptorError::ReturnTypeError(
                DescriptorTypeError::InvalidTypeOpener
            ))
        );
        assert_eq!(
            MethodDescriptor::parse("()V"),
            Ok(MethodDescriptor {
                parameter_types: Vec::new(),
                return_type: None,
            })
        );
        assert_eq!(
            MethodDescriptor::parse("(I)V"),
            Ok(MethodDescriptor {
                parameter_types: vec![DescriptorTypeBasic::Int.into()],
                return_type: None,
            })
        );
        assert_eq!(
            MethodDescriptor::parse("(IDJ)V"),
            Ok(MethodDescriptor {
                parameter_types: vec![
                    DescriptorTypeBasic::Int.into(),
                    DescriptorTypeBasic::Double.into(),
                    DescriptorTypeBasic::Long.into()
                ],
                return_type: None,
            })
        );
        assert_eq!(
            MethodDescriptor::parse("(IDLjava/lang/Thread;)Ljava/lang/Object;"),
            Ok(MethodDescriptor {
                parameter_types: vec![
                    DescriptorTypeBasic::Int.into(),
                    DescriptorTypeBasic::Double.into(),
                    DescriptorTypeBasic::ClassName("java/lang/Thread".into()).into()
                ],
                return_type: Some(DescriptorTypeBasic::ClassName("java/lang/Object".into()).into()),
            })
        );
    }
}
