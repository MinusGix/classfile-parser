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
    pub fn parse(text: &'a str) -> Result<MethodDescriptor<'a>, MethodDescriptorError> {
        // It may or may not be more efficient to inline these iterations
        // but this avoid duplicating parsing code.
        let mut iter = MethodDescriptor::parse_iter(text)?;
        let mut parameter_types = Vec::new();
        while let Some(parameter) = iter.next() {
            parameter_types.push(parameter?);
        }

        let return_type = iter.finish_return_type()?;

        Ok(MethodDescriptor {
            parameter_types,
            return_type,
        })
    }

    pub fn parse_iter(
        text: &'a str,
    ) -> Result<MethodDescriptorParserIterator<'a>, MethodDescriptorError> {
        MethodDescriptorParserIterator::new(text)
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

/// Parses the descriptor types as an iterator
/// Note: If you want the return type, then you have to call `finish_return_type`
#[derive(Clone)]
pub struct MethodDescriptorParserIterator<'a> {
    text: &'a str,
    got_all_parameters: bool,
    errored: bool,
    processed_parameters: usize,
}
impl<'a> MethodDescriptorParserIterator<'a> {
    fn new(text: &'a str) -> Result<MethodDescriptorParserIterator<'a>, MethodDescriptorError> {
        if text.is_empty() {
            return Err(MethodDescriptorError::Empty);
        }

        if !text.starts_with('(') {
            return Err(MethodDescriptorError::NoOpeningBracket);
        }

        let text = &text[1..];

        Ok(MethodDescriptorParserIterator {
            text,
            got_all_parameters: false,
            errored: false,
            processed_parameters: 0,
        })
    }

    pub fn finish_return_type(self) -> Result<Option<DescriptorType<'a>>, MethodDescriptorError> {
        let ch = self.text.chars().next();
        if let Some(ch) = ch {
            // Void is the expected type for returning nothing, but we transform it into `None`
            if ch == 'V' {
                Ok(None)
            } else {
                // Otherwise, we try parsing it as a type
                let (typ, after_text) = DescriptorType::parse(self.text)
                    .map_err(MethodDescriptorError::ReturnTypeError)?;
                if !after_text.is_empty() {
                    // There was unhandled remaining data, which means it was bad or that this parsing code is incorrect
                    return Err(MethodDescriptorError::RemainingData);
                }

                Ok(Some(typ))
            }
        } else {
            // This is distinct from having a void return type
            Err(MethodDescriptorError::NoReturnType)
        }
    }
}
impl<'a> Iterator for MethodDescriptorParserIterator<'a> {
    type Item = Result<DescriptorType<'a>, MethodDescriptorError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.got_all_parameters || self.errored {
            return None;
        }

        let is_descriptor_type = self
            .text
            .chars()
            .next()
            .map(DescriptorType::is_beginning_char)
            .unwrap_or(false);
        if !is_descriptor_type {
            self.got_all_parameters = true;
            if !self.text.starts_with(')') {
                self.errored = true;
                return Some(Err(MethodDescriptorError::NoClosingBracket));
            }

            // Skip ')'
            self.text = &self.text[1..];

            None
        } else {
            let res = DescriptorType::parse(self.text).map_err(|x| {
                MethodDescriptorError::ParameterTypeError(x, self.processed_parameters)
            });
            match res {
                Ok((parameter, after_text)) => {
                    self.text = after_text;
                    self.processed_parameters += 1;
                    Some(Ok(parameter))
                }
                Err(err) => {
                    self.errored = true;
                    Some(Err(err))
                }
            }
        }
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
