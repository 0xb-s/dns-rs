use std::io::{self, Read, Write};

#[derive(Debug, Clone)]
pub struct Question {
    pub qname: String,
    pub qtype: u16,
    pub qclass: u16,
}

impl Question {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        let qname = read_qname(reader)?;
        let mut buf = [0u8; 4];
        reader.read_exact(&mut buf)?;
        Ok(Question {
            qname,
            qtype: u16::from_be_bytes([buf[0], buf[1]]),
            qclass: u16::from_be_bytes([buf[2], buf[3]]),
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_qname(writer, &self.qname)?;
        writer.write_all(&self.qtype.to_be_bytes())?;
        writer.write_all(&self.qclass.to_be_bytes())?;
        Ok(())
    }
}

fn read_qname<R: Read>(reader: &mut R) -> io::Result<String> {
    let mut labels = Vec::new();
    loop {
        let len = {
            let mut len_buf = [0u8; 1];
            reader.read_exact(&mut len_buf)?;
            len_buf[0]
        };
        if len == 0 {
            break;
        }

        let mut label = vec![0u8; len as usize];
        reader.read_exact(&mut label)?;
        labels.push(
            String::from_utf8(label)
                .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid label"))?,
        );
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
