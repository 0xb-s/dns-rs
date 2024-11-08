pub mod errors;
pub mod parser;

use crate::errors::ZoneParserError;
use dns_core::record::Record;
use parser::ZoneParser;

pub struct ZoneFile {
    pub records: Vec<Record>,
}

impl ZoneFile {
    pub fn parse(file_path: &str) -> Result<Self, ZoneParserError> {
        let mut parser = ZoneParser::new();
        let records = parser.parse_zone_file(file_path)?;
        Ok(ZoneFile { records })
    }
}
