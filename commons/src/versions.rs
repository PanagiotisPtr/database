use anyhow::{format_err, Result};

const V0_BYTES: &'static [u8] = &0u16.to_le_bytes();

#[derive(PartialEq)]
pub enum Version {
    V0,
}

impl Version {
    pub fn to_bytes(&self) -> &'static [u8] {
        match self {
            Version::V0 => V0_BYTES,
        }
    }
}

pub fn from_bytes(bytes: [u8; 2]) -> Result<Version> {
    match u16::from_le_bytes(bytes) {
        0 => Ok(Version::V0),
        _ => Err(format_err!("unknown version")),
    }
}

pub fn from<R>(reader: &mut R) -> Result<Version>
where
    R: std::io::Read,
{
    let mut bytes: [u8; 2] = [0, 0];
    reader.read_exact(&mut bytes)?;
    from_bytes(bytes)
}
