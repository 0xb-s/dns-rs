use std::fmt;

#[derive(Debug)]
pub enum ZoneParserError {
    IoError(std::io::Error),
    RegexError(regex::Error),
    InvalidDirective(String),
    UnknownDirective(String),
    InvalidRecord(String),
    Utf8Error(std::string::FromUtf8Error),
    ParseIntError(std::num::ParseIntError),
}

impl fmt::Display for ZoneParserError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ZoneParserError::IoError(e) => write!(f, "IO Error: {}", e),
            ZoneParserError::RegexError(e) => write!(f, "Regex Error: {}", e),
            ZoneParserError::InvalidDirective(e) => write!(f, "Invalid Directive: {}", e),
            ZoneParserError::UnknownDirective(e) => write!(f, "Unknown Directive: {}", e),
            ZoneParserError::InvalidRecord(e) => write!(f, "Invalid Record: {}", e),
            ZoneParserError::Utf8Error(e) => write!(f, "UTF-8 Error: {}", e),
            ZoneParserError::ParseIntError(e) => write!(f, "Parse Int Error: {}", e),
        }
    }
}

impl From<std::io::Error> for ZoneParserError {
    fn from(error: std::io::Error) -> Self {
        ZoneParserError::IoError(error)
    }
}

impl From<regex::Error> for ZoneParserError {
    fn from(error: regex::Error) -> Self {
        ZoneParserError::RegexError(error)
    }
}

impl From<std::string::FromUtf8Error> for ZoneParserError {
    fn from(error: std::string::FromUtf8Error) -> Self {
        ZoneParserError::Utf8Error(error)
    }
}

impl From<std::num::ParseIntError> for ZoneParserError {
    fn from(error: std::num::ParseIntError) -> Self {
        ZoneParserError::ParseIntError(error)
    }
}
