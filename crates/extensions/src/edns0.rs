use std::{error::Error as Errr, io::Read};

use dns_core::{message::Message, RData, Record};
use serde::ser::SerializeStruct;
use serde::{
    de::{MapAccess, Visitor},
    Deserialize, Deserializer, Serialize, Serializer,
};
use thiserror::Error;

pub struct EDNS0 {
    pub version: u8,
    pub flags: u16,
    pub udp_size: u16,
    pub ext_rcode: u8,
    pub edns0_data: Vec<u8>,
}
impl std::fmt::Debug for EDNS0 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("EDNS0")
            .field("version", &self.version)
            .field("flags", &self.flags)
            .field("udp_size", &self.udp_size)
            .field("ext_rcode", &self.ext_rcode)
            .field("edns0_data", &self.edns0_data)
            .finish()
    }
}
impl Serialize for EDNS0 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("EDNS0", 5)?;
        state.serialize_field("version", &self.version)?;
        state.serialize_field("flags", &self.flags)?;
        state.serialize_field("udp_size", &self.udp_size)?;
        state.serialize_field("ext_rcode", &self.ext_rcode)?;
        state.serialize_field("edns0_data", &self.edns0_data)?;
        state.end()
    }
}
impl<'de> Deserialize<'de> for EDNS0 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        enum Field {
            Version,
            Flags,
            UdpSize,
            ExtRcode,
            Edns0Data,
        }

        struct EDNS0Visitor;

        impl<'de> Visitor<'de> for EDNS0Visitor {
            type Value = EDNS0;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("struct EDNS0")
            }

            fn visit_map<V>(self, mut map: V) -> Result<EDNS0, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut version = None;
                let mut flags = None;
                let mut udp_size = None;
                let mut ext_rcode = None;
                let mut edns0_data = None;

                while let Some(key) = map.next_value().unwrap() {
                    match key {
                        Field::Version => version = Some(map.next_value()?),
                        Field::Flags => flags = Some(map.next_value()?),
                        Field::UdpSize => udp_size = Some(map.next_value()?),
                        Field::ExtRcode => ext_rcode = Some(map.next_value()?),
                        Field::Edns0Data => edns0_data = Some(map.next_value()?),
                    }
                }

                let version = version.ok_or_else(|| serde::de::Error::missing_field("version"))?;
                let flags = flags.ok_or_else(|| serde::de::Error::missing_field("flags"))?;
                let udp_size =
                    udp_size.ok_or_else(|| serde::de::Error::missing_field("udp_size"))?;
                let ext_rcode =
                    ext_rcode.ok_or_else(|| serde::de::Error::missing_field("ext_rcode"))?;
                let edns0_data =
                    edns0_data.ok_or_else(|| serde::de::Error::missing_field("edns0_data"))?;

                Ok(EDNS0 {
                    version,
                    flags,
                    udp_size,
                    ext_rcode,
                    edns0_data,
                })
            }
        }

        const FIELDS: &[&str] = &["version", "flags", "udp_size", "ext_rcode", "edns0_data"];
        deserializer.deserialize_struct("EDNS0", FIELDS, EDNS0Visitor)
    }
}
impl Clone for EDNS0 {
    fn clone(&self) -> Self {
        EDNS0 {
            version: self.version,
            flags: self.flags,
            udp_size: self.udp_size,
            ext_rcode: self.ext_rcode,
            edns0_data: self.edns0_data.clone(),
        }
    }
}
impl EDNS0 {
    pub fn new() -> Self {
        EDNS0 {
            version: 0,
            flags: 0,
            udp_size: 4096,
            ext_rcode: 0,
            edns0_data: Vec::new(),
        }
    }

    pub fn parse(message: &Message) -> Result<Option<Self>, EDNS0Error> {
        for additional in &message.additionals {
            if additional.rtype == 41 {
  
                if let RData::Raw(data) = &additional.rdata {
                    let mut reader = &data[..];
                    let mut edns0 = EDNS0::new();
                    edns0.version = read_u8(&mut reader)?;
                    edns0.flags = read_u16(&mut reader)?;
                    edns0.udp_size = read_u16(&mut reader)?;
                    edns0.ext_rcode = read_u8(&mut reader)?;
                    let rdlength = data.len() - 6;
                    if rdlength > 0 {
                        let mut edns0_data = vec![0u8; rdlength];
                        reader.read_exact(&mut edns0_data)?;
                        edns0.edns0_data = edns0_data;
                    }
                    return Ok(Some(edns0));
                }
            }
        }
        Ok(None)
    }

    pub fn add_to_message(&self, message: &mut Message) -> Result<(), Box<dyn Errr>> {
        let mut data = Vec::with_capacity(6 + self.edns0_data.len());
        data.push(self.version);
        data.extend_from_slice(&self.flags.to_be_bytes());
        data.extend_from_slice(&self.udp_size.to_be_bytes());
        data.push(self.ext_rcode);
        data.extend_from_slice(&self.edns0_data);

        message.write(&mut data)?;
        let opt_record = Record {
            name: ".".to_string(),
            rtype: 41,
            rclass: self.udp_size,
            ttl: ((self.flags as u32) << 16) | (self.ext_rcode as u32),
            rdata: RData::Raw(data.to_vec()),
        };
        message.additionals.push(opt_record);
        Ok(())
    }
}
#[derive(Debug, Error)]
pub enum EDNS0Error {
    #[error("IO Error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Invalid EDNS0 record")]
    InvalidRecord,
}

//todo move this to helper
fn read_u8<R: Read>(reader: &mut R) -> std::io::Result<u8> {
    let mut buf = [0u8; 1];
    reader.read_exact(&mut buf)?;
    Ok(buf[0])
}

fn read_u16<R: Read>(reader: &mut R) -> std::io::Result<u16> {
    let mut buf = [0u8; 2];
    reader.read_exact(&mut buf)?;
    Ok(u16::from_be_bytes(buf))
}
