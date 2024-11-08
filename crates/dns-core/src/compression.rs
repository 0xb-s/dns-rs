use std::collections::HashMap;
use std::io::{self, Read, Seek, Write};

pub fn compress_name<W: Write + Seek>(
    writer: &mut W,
    name: &str,
    compression_map: &mut HashMap<String, u16>,
) -> io::Result<()> {
    let labels: Vec<&str> = name.split('.').collect();
    let mut current = String::new();
    for label in labels.iter() {
        if !current.is_empty() {
            current.push('.');
        }
        current.push_str(label);
        if let Some(&offset) = compression_map.get(&current) {
            // Write pointer
            let pointer = 0xC000 | offset;
            writer.write_all(&pointer.to_be_bytes())?;
            return Ok(());
        } else {
            compression_map.insert(
                current.clone(),
                writer.seek(io::SeekFrom::Current(0))? as u16,
            );
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
    }
    writer.write_all(&[0u8])?;
    Ok(())
}

pub fn decompress_name<R: Read>(
    reader: &mut R,
    compression_map: &HashMap<u16, String>,
) -> io::Result<String> {
    let mut name = String::new();

    loop {
        let len = {
            let mut len_buf = [0u8; 1];
            reader.read_exact(&mut len_buf)?;
            len_buf[0]
        };
        if len == 0 {
            break;
        }
        if (len & 0xC0) == 0xC0 {
            let mut pointer_buf = [0u8; 1];
            reader.read_exact(&mut pointer_buf)?;
            let ptr = ((len & 0x3F) as u16) << 8 | (pointer_buf[0] as u16);
            if let Some(p) = compression_map.get(&ptr) {
                if !name.is_empty() {
                    name.push('.');
                }
                name.push_str(p);
                break;
            } else {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Invalid compression pointer",
                ));
            }
        } else {
            let mut label = vec![0u8; len as usize];
            reader.read_exact(&mut label)?;
            if !name.is_empty() {
                name.push('.');
            }
            name.push_str(
                &String::from_utf8(label)
                    .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid label"))?,
            );
        }
    }
    Ok(name)
}
