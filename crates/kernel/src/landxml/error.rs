use std::fmt::{Display, Formatter};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LandXmlErrorKind {
    Parse,
    InvalidInput,
    NotFound,
    OutOfRange,
}

#[derive(Clone, Debug)]
pub struct LandXmlError {
    pub kind: LandXmlErrorKind,
    pub message: String,
}

impl LandXmlError {
    pub fn parse(message: impl Into<String>) -> Self {
        Self {
            kind: LandXmlErrorKind::Parse,
            message: message.into(),
        }
    }

    pub fn invalid_input(message: impl Into<String>) -> Self {
        Self {
            kind: LandXmlErrorKind::InvalidInput,
            message: message.into(),
        }
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self {
            kind: LandXmlErrorKind::NotFound,
            message: message.into(),
        }
    }

    pub fn out_of_range(message: impl Into<String>) -> Self {
        Self {
            kind: LandXmlErrorKind::OutOfRange,
            message: message.into(),
        }
    }
}

impl Display for LandXmlError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for LandXmlError {}
