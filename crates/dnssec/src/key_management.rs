
use ring::rand::SystemRandom;
use ring::signature::{EcdsaKeyPair, Ed25519KeyPair, KeyPair, RsaKeyPair};
use std::error::Error;
use std::fs;
use std::io::{self, Write};
use std::path::Path;

pub enum KeyType {
    RSA,
    ECDSA,
    Ed25519,
}
#[derive(Debug, Clone)]
pub struct RRSig {
    pub type_covered: u16,
    pub algorithm: u8,
    pub labels: u8,
    pub original_ttl: u32,
    pub expiration: u32,
    pub inception: u32,
    pub key_tag: u16,
    pub signer_name: String,
    pub signature: Vec<u8>,
}

impl RRSig {
    pub fn new(
        type_covered: u16,
        algorithm: u8,
        labels: u8,
        original_ttl: u32,
        expiration: u32,
        inception: u32,
        key_tag: u16,
        signer_name: String,
        signature: Vec<u8>,
    ) -> Self {
        RRSig {
            type_covered,
            algorithm,
            labels,
            original_ttl,
            expiration,
            inception,
            key_tag,
            signer_name,
            signature,
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Box<dyn Error>> {
        if bytes.len() < 18 {
            return Err("RRSIG record too short".into());
        }

        let type_covered = u16::from_be_bytes(bytes[0..2].try_into()?);
        let algorithm = bytes[2];
        let labels = bytes[3];
        let original_ttl = u32::from_be_bytes(bytes[4..8].try_into()?);
        let expiration = u32::from_be_bytes(bytes[8..12].try_into()?);
        let inception = u32::from_be_bytes(bytes[12..16].try_into()?);
        let key_tag = u16::from_be_bytes(bytes[16..18].try_into()?);

        let signer_name_end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
        let signer_name = String::from_utf8(bytes[18..signer_name_end].to_vec())?;

    
        let signature = bytes[signer_name_end + 1..].to_vec();

        Ok(Self {
            type_covered,
            algorithm,
            labels,
            original_ttl,
            expiration,
            inception,
            key_tag,
            signer_name,
            signature,
        })
    }
}

pub struct DNSSECKey {
    pub key_type: KeyType,
    pub private_key: Vec<u8>,
    pub public_key: Vec<u8>,
    pub key_tag: u16,
    pub algorithm: u8,
}

impl DNSSECKey {
    pub fn generate(key_type: KeyType) -> io::Result<Self> {
        match key_type {
            KeyType::Ed25519 => {
                let rng = SystemRandom::new();
                let pkcs8_bytes = ring::signature::Ed25519KeyPair::generate_pkcs8(&rng)
                    .map_err(|_| io::Error::new(io::ErrorKind::Other, "Key generation failed"))?;
                let key_pair = Ed25519KeyPair::from_pkcs8(pkcs8_bytes.as_ref())
                    .map_err(|_| io::Error::new(io::ErrorKind::Other, "Invalid key"))?;
                let pkcs8_document = Ed25519KeyPair::generate_pkcs8(&rng).unwrap();

                Ok(DNSSECKey {
                    key_type,
                    private_key: pkcs8_document.as_ref().to_vec(),
                    public_key: key_pair.public_key().as_ref().to_vec(),
                    key_tag: calculate_key_tag(&key_pair.public_key().as_ref()),
                    algorithm: 13, 
                })
            }
       
            _ => unimplemented!(),
        }
    }

    pub fn save_to_file(&self, path: &Path) -> io::Result<()> {
        let mut file = fs::File::create(path)?;
        file.write_all(&self.private_key)?;
        Ok(())
    }


}

fn calculate_key_tag(pub_key: &[u8]) -> u16 {

    let mut ac = 0u32;
    for (i, byte) in pub_key.iter().enumerate() {
        if i & 1 == 0 {
            ac += ((*byte as u32) << 8);
        } else {
            ac += *byte as u32;
        }
    }
    ((ac + ((ac >> 16) & 0xFFFF)) & 0xFFFF) as u16
}
