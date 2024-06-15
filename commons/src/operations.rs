use anyhow::{format_err, Result};

const OPERATION_PING_BYTES: &'static [u8] = &0u16.to_le_bytes();
const OPERATION_EXIT_BYTES: &'static [u8] = &1u16.to_le_bytes();
const OPERATION_GET_BYTES: &'static [u8] = &2u16.to_le_bytes();
const OPERATION_SET_BYTES: &'static [u8] = &3u16.to_le_bytes();
const OPERATION_DEL_BYTES: &'static [u8] = &4u16.to_le_bytes();

#[derive(PartialEq)]
pub enum Operation {
    PING,
    EXIT,
    GET,
    SET,
    DEL,
}

impl Operation {
    pub fn to_bytes(&self) -> &[u8] {
        match self {
            Operation::PING => OPERATION_PING_BYTES,
            Operation::EXIT => OPERATION_EXIT_BYTES,
            Operation::GET => OPERATION_GET_BYTES,
            Operation::SET => OPERATION_SET_BYTES,
            Operation::DEL => OPERATION_DEL_BYTES,
        }
    }
}

pub fn from_bytes(bytes: [u8; 2]) -> Result<Operation> {
    match u16::from_le_bytes(bytes) {
        0 => Ok(Operation::PING),
        1 => Ok(Operation::EXIT),
        2 => Ok(Operation::GET),
        3 => Ok(Operation::SET),
        4 => Ok(Operation::DEL),
        _ => Err(format_err!("unknown operation")),
    }
}

pub fn from<R>(reader: &mut R) -> Result<Operation>
where
    R: std::io::Read,
{
    let mut bytes: [u8; 2] = [0, 0];
    reader.read_exact(&mut bytes)?;
    from_bytes(bytes)
}
