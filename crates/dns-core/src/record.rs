
use std::io::{self, Read, Write};
use std::net::{Ipv4Addr, Ipv6Addr};
use thiserror::Error;

#[derive(Debug, Clone)]
pub enum RData {
    A(Ipv4Addr),
    AAAA(Ipv6Addr),
    CNAME(String),
    MX {
        preference: u16,
        exchange: String,
    },
    NS(String),
    SOA {
        mname: String,
        rname: String,
        serial: u32,
        refresh: u32,
        retry: u32,
        expire: u32,
        minimum: u32,
    },
    TXT(String),
    Raw(Vec<u8>), // For unsupported or unknown types
}

#[derive(Debug, Clone)]
pub struct Record {
    pub name: String,
    pub rtype: u16,
    pub rclass: u16,
    pub ttl: u32,
    pub rdata: RData,
}

impl Record {
    pub fn read<R: Read>(reader: &mut R) -> Result<Self, RecordError> {
        let name = read_qname(reader)?;
        let rtype = read_u16(reader).unwrap();
        let rclass = read_u16(reader).unwrap();
        let ttl = read_u32(reader).unwrap();
        let rdlength = read_u16(reader).unwrap();
        let mut rdata_buf = vec![0u8; rdlength as usize];
        reader.read_exact(&mut rdata_buf).unwrap();

        let rdata = match rtype {
            1 => {
                // A
                if rdlength != 4 {
                    return Err(RecordError::InvalidRDataLength(rtype));
                }
                let ip = Ipv4Addr::new(rdata_buf[0], rdata_buf[1], rdata_buf[2], rdata_buf[3]);
                RData::A(ip)
            }
            28 => {
                // AAAA
                if rdlength != 16 {
                    return Err(RecordError::InvalidRDataLength(rtype));
                }
                let ip = Ipv6Addr::from([
                    rdata_buf[0],
                    rdata_buf[1],
                    rdata_buf[2],
                    rdata_buf[3],
                    rdata_buf[4],
                    rdata_buf[5],
                    rdata_buf[6],
                    rdata_buf[7],
                    rdata_buf[8],
                    rdata_buf[9],
                    rdata_buf[10],
                    rdata_buf[11],
                    rdata_buf[12],
                    rdata_buf[13],
                    rdata_buf[14],
                    rdata_buf[15],
                ]);
                RData::AAAA(ip)
            }
            5 => {
                // CNAME
                let mut cursor = io::Cursor::new(&rdata_buf);
                let cname = read_qname(&mut cursor)?;
                RData::CNAME(cname)
            }
            15 => {
                // MX
                if rdlength < 3 {
                    return Err(RecordError::InvalidRDataLength(rtype));
                }
                let preference = ((rdata_buf[0] as u16) << 8) | (rdata_buf[1] as u16);
                let mut cursor = io::Cursor::new(&rdata_buf[2..]);
                let exchange = read_qname(&mut cursor)?;
                RData::MX {
                    preference,
                    exchange,
                }
            }
            2 => {
                // NS
                let mut cursor = io::Cursor::new(&rdata_buf);
                let ns = read_qname(&mut cursor)?;
                RData::NS(ns)
            }
            6 => {
                // SOA
                let mut cursor = io::Cursor::new(&rdata_buf);
                let mname = read_qname(&mut cursor)?;
                let rname = read_qname(&mut cursor)?;
                let serial = read_u32(&mut cursor).unwrap();
                let refresh = read_u32(&mut cursor).unwrap();
                let retry = read_u32(&mut cursor).unwrap();
                let expire = read_u32(&mut cursor).unwrap();
                let minimum = read_u32(&mut cursor).unwrap();
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
            16 => {
               
                let txt =
                    String::from_utf8(rdata_buf.clone()).map_err(|_| RecordError::InvalidUTF8)?;
                RData::TXT(txt)
            }
            _ => RData::Raw(rdata_buf),
        };
        Ok(Record {
            name,
            rtype,
            rclass,
            ttl,
            rdata,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_qname(writer, &self.name)?;
        writer.write_all(&self.rtype.to_be_bytes())?;
        writer.write_all(&self.rclass.to_be_bytes())?;
        writer.write_all(&self.ttl.to_be_bytes())?;
        let rdata_bytes = match &self.rdata {
            RData::A(ip) => ip.octets().to_vec(),
            RData::AAAA(ip) => ip.octets().to_vec(),
            RData::CNAME(cname) => {
                let mut buf = Vec::new();
                write_qname(&mut buf, cname)?;
                buf
            }
            RData::MX {
                preference,
                exchange,
            } => {
                let mut buf = Vec::new();
                buf.extend(&preference.to_be_bytes());
                write_qname(&mut buf, exchange)?;
                buf
            }
            RData::NS(ns) => {
                let mut buf = Vec::new();
                write_qname(&mut buf, ns)?;
                buf
            }
            RData::SOA {
                mname,
                rname,
                serial,
                refresh,
                retry,
                expire,
                minimum,
            } => {
                let mut buf = Vec::new();
                write_qname(&mut buf, mname)?;
                write_qname(&mut buf, rname)?;
                buf.extend(&serial.to_be_bytes());
                buf.extend(&refresh.to_be_bytes());
                buf.extend(&retry.to_be_bytes());
                buf.extend(&expire.to_be_bytes());
                buf.extend(&minimum.to_be_bytes());
                buf
            }
            RData::TXT(txt) => {
                let mut buf = Vec::new();
                buf.extend(&(txt.len() as u8).to_be_bytes());
                buf.extend(txt.as_bytes());
                buf
            }
            RData::Raw(data) => data.clone(),
        };
        writer.write_all(&(rdata_bytes.len() as u16).to_be_bytes())?;
        writer.write_all(&rdata_bytes)?;
        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum RecordError {
    #[error("Invalid RData length for type {0}")]
    InvalidRDataLength(u16),
    #[error("Invalid UTF-8 in RData")]
    InvalidUTF8,
    #[error("IO Error: {0}")]
    IoError(#[from] io::Error),
}

fn read_qname<R: Read>(reader: &mut R) -> Result<String, RecordError> {
    let mut labels = Vec::new();
    loop {
        let len = {
            let mut len_buf = [0u8; 1];
            reader.read_exact(&mut len_buf).unwrap();
            len_buf[0]
        };
        if len == 0 {
            break;
        }
        let mut label = vec![0u8; len as usize];
        reader.read_exact(&mut label).unwrap();
        labels.push(String::from_utf8(label).map_err(|_| RecordError::InvalidUTF8)?);
    }
    Ok(labels.join("."))
}

fn write_qname<W: Write>(writer: &mut W, qname: &str) -> io::Result<()> {
    for label in qname.split('.') {
        let len = label.len();
        if len > 63 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Label too long",
            ));
        }
        writer.write_all(&[len as u8])?;
        writer.write_all(label.as_bytes())?;
    }
    writer.write_all(&[0u8])?;
    Ok(())
}

fn read_u16<R: Read>(reader: &mut R) -> Result<u16, io::Error> {
    let mut buf = [0u8; 2];
    reader.read_exact(&mut buf)?;
    Ok(u16::from_be_bytes(buf))
}

fn read_u32<R: Read>(reader: &mut R) -> Result<u32, io::Error> {
    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf)?;
    Ok(u32::from_be_bytes(buf))
}
