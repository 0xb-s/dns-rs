// zone-parser/src/parser.rs

use crate::errors::ZoneParserError;
use dns_core::record::{RData, Record};
use regex::Regex;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct ZoneParser {
    origin: String,
    ttl: u32,
}

impl ZoneParser {
    pub fn new() -> Self {
        ZoneParser {
            origin: String::new(),
            ttl: 3600, // Default TTL
        }
    }

    pub fn parse_zone_file(&mut self, file_path: &str) -> Result<Vec<Record>, ZoneParserError> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut records = Vec::new();

        for line in reader.lines() {
            let line = line?;
            let line = line.trim();

            // Skip comments and empty lines
            if line.is_empty() || line.starts_with(';') {
                continue;
            }

            if line.starts_with('$') {
                self.handle_directive(line, Path::new(file_path))?;
                continue;
            }

      
            let record = self.parse_record(line)?;
            records.push(record);
        }

        Ok(records)
    }

    fn handle_directive(&mut self, line: &str, current_path: &Path) -> Result<(), ZoneParserError> {
        let tokens: Vec<&str> = line.split_whitespace().collect();
        match tokens[0].to_uppercase().as_str() {
            "$ORIGIN" => {
                if tokens.len() < 2 {
                    return Err(ZoneParserError::InvalidDirective(format!(
                        "$ORIGIN missing value: {}",
                        line
                    )));
                }
                self.origin = tokens[1].trim_end_matches('.').to_string();
            }
            "$TTL" => {
                if tokens.len() < 2 {
                    return Err(ZoneParserError::InvalidDirective(format!(
                        "$TTL missing value: {}",
                        line
                    )));
                }
                self.ttl = tokens[1].parse().map_err(|_| {
                    ZoneParserError::InvalidDirective(format!("Invalid TTL value: {}", tokens[1]))
                })?;
            }
            "$INCLUDE" => {
                if tokens.len() < 2 {
                    return Err(ZoneParserError::InvalidDirective(format!(
                        "$INCLUDE missing file path: {}",
                        line
                    )));
                }
                let include_path = current_path
                    .parent()
                    .unwrap_or_else(|| Path::new("."))
                    .join(tokens[1]);
                let _included_records = self.parse_zone_file(include_path.to_str().unwrap())?;
            }
            "$GENERATE" => {
                todo!()
            }
            _ => {
                return Err(ZoneParserError::UnknownDirective(format!(
                    "Unknown directive: {}",
                    tokens[0]
                )));
            }
        }
        Ok(())
    }
//rewrite better impl here
    fn parse_record(&self, line: &str) -> Result<Record, ZoneParserError> {
        let re = Regex::new(
            r"^(?P<name>\S+)\s+(?P<ttl>\d+)?\s+(?P<class>\S+)?\s+(?P<type>\S+)\s+(?P<data>.+)$",
        )
        .unwrap();
        if let Some(caps) = re.captures(line) {
            let name = caps.name("name").unwrap().as_str().to_string();
            let ttl = caps
                .name("ttl")
                .map_or(self.ttl, |m| m.as_str().parse().unwrap_or(self.ttl));
            let class = caps
                .name("class")
                .map_or("IN".to_string(), |m| m.as_str().to_string());
            let record_type = caps.name("type").unwrap().as_str();
            let data = caps.name("data").unwrap().as_str();

            let rtype = match record_type.to_uppercase().as_str() {
                "A" => 1,
                "AAAA" => 28,
                "CNAME" => 5,
                "MX" => 15,
                "NS" => 2,
                "SOA" => 6,
                "TXT" => 16,
                _ => 255,
            };

            let rdata = match record_type.to_uppercase().as_str() {
                "A" => {
                    let ip: std::net::Ipv4Addr = data.parse().unwrap();
                    RData::A(ip)
                }
                "AAAA" => {
                    let ip: std::net::Ipv6Addr = data.parse().unwrap();
                    RData::AAAA(ip)
                }
                "CNAME" => RData::CNAME(data.to_string()),
                "MX" => {
                
                    let parts: Vec<&str> = data.split_whitespace().collect();
                    let preference: u16 = parts[0].parse()?;
                    let exchange = parts[1].to_string();
                    RData::MX {
                        preference,
                        exchange,
                    }
                }
                "NS" => RData::NS(data.to_string()),
                "SOA" => {
               
                    let parts: Vec<&str> = data.split_whitespace().collect();
                    let mname = parts[0].to_string();
                    let rname = parts[1].to_string();
                    let serial: u32 = parts[2].parse()?;
                    let refresh: u32 = parts[3].parse()?;
                    let retry: u32 = parts[4].parse()?;
                    let expire: u32 = parts[5].parse()?;
                    let minimum: u32 = parts[6].parse()?;
                    RData::SOA {
                        mname,
                        rname,
                        serial,
                        refresh,
                        retry,
                        expire,
                        minimum,
                    }
                }
                "TXT" => RData::TXT(data.to_string()),
                _ => RData::Raw(data.as_bytes().to_vec()),
            };

            Ok(Record {
                name,
                rtype,
                rclass: match class.as_str() {
                    "IN" => 1,
                    _ => 255,
                },
                ttl,
                rdata,
            })
        } else {
            Err(ZoneParserError::InvalidRecord(format!(
                "Failed to parse record: {}",
                line
            )))
        }
    }
}
