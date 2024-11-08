use std::io::{self, Read, Write};

#[derive(Debug, Clone)]
pub struct Header {
    pub id: u16,
    pub qr: bool,
    pub opcode: u8,
    pub aa: bool,
    pub tc: bool,
    pub rd: bool,
    pub ra: bool,
    pub z: u8,
    pub rcode: u8,
    pub qdcount: u16,
    pub ancount: u16,
    pub nscount: u16,
    pub arcount: u16,
}

impl Header {
    pub fn new() -> Self {
        Header {
            id: 0,
            qr: false,
            opcode: 0,
            aa: false,
            tc: false,
            rd: false,
            ra: false,
            z: 0,
            rcode: 0,
            qdcount: 0,
            ancount: 0,
            nscount: 0,
            arcount: 0,
        }
    }

    // Methods to parse and write headers
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        let mut buf = [0u8; 12];
        reader.read_exact(&mut buf)?;
        Ok(Header {
            id: u16::from_be_bytes([buf[0], buf[1]]),
            qr: (buf[2] & 0x80) != 0,
            opcode: (buf[2] & 0x78) >> 3,
            aa: (buf[2] & 0x04) != 0,
            tc: (buf[2] & 0x02) != 0,
            rd: (buf[2] & 0x01) != 0,
            ra: (buf[3] & 0x80) != 0,
            z: (buf[3] & 0x70) >> 4,
            rcode: buf[3] & 0x0F,
            qdcount: u16::from_be_bytes([buf[4], buf[5]]),
            ancount: u16::from_be_bytes([buf[6], buf[7]]),
            nscount: u16::from_be_bytes([buf[8], buf[9]]),
            arcount: u16::from_be_bytes([buf[10], buf[11]]),
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_all(&self.id.to_be_bytes())?;
        let mut buf2 = 0u8;
        buf2 |= (self.qr as u8) << 7;
        buf2 |= (self.opcode & 0x0F) << 3;
        buf2 |= (self.aa as u8) << 2;
        buf2 |= (self.tc as u8) << 1;
        buf2 |= self.rd as u8;
        writer.write_all(&[buf2])?;

        let mut buf3 = 0u8;
        buf3 |= (self.ra as u8) << 7;
        buf3 |= (self.z & 0x07) << 4;
        buf3 |= self.rcode & 0x0F;
        writer.write_all(&[buf3])?;

        writer.write_all(&self.qdcount.to_be_bytes())?;
        writer.write_all(&self.ancount.to_be_bytes())?;
        writer.write_all(&self.nscount.to_be_bytes())?;
        writer.write_all(&self.arcount.to_be_bytes())?;
        Ok(())
    }
}
