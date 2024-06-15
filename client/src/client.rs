use crate::command::Command;
use anyhow::Result;
use commons::{
    messages::{GetRequest, GetResponse, SetRequest, SetResponse},
    operations::{self, Operation},
    versions::{self, Version},
};
use rand::{rngs::ThreadRng, Rng};
use serde::{de::DeserializeOwned, Serialize};
use std::{
    io::{Read, Write},
    net::TcpStream,
};

pub struct Client {
    version: Version,
    stream: TcpStream,
    rng: ThreadRng,
}

impl Client {
    pub fn new(addr: &str) -> Result<Self> {
        let stream = TcpStream::connect(addr)?;
        let rng = rand::thread_rng();

        Ok(Client {
            version: Version::V0,
            stream,
            rng,
        })
    }

    fn get_header_size(&self) -> usize {
        2 + 4 + 2
    }

    fn put_message(&mut self, operation: Operation, data: &[u8]) -> Result<u32> {
        let id = self.rng.gen::<u32>();
        let mut buffer = Vec::with_capacity(self.get_header_size());
        // message version
        buffer.write_all(self.version.to_bytes())?;
        // message id
        buffer.write_all(&id.to_le_bytes())?;
        // operation
        buffer.write_all(operation.to_bytes())?;
        // data size
        buffer.write_all(&data.len().to_le_bytes())?;
        // data
        buffer.write_all(&data)?;

        self.stream.write_all(&buffer)?;

        Ok(id)
    }

    fn get_message(&mut self, id: u32, operation: Operation) -> Result<Vec<u8>> {
        let version = versions::from(&mut self.stream)?;
        if version != self.version {
            return Err(anyhow::format_err!("mismatched versions"));
        }

        let mut id_buff = 0u32.to_le_bytes();
        self.stream.read_exact(&mut id_buff)?;
        let m_id = u32::from_le_bytes(id_buff);
        if m_id != id {
            return Err(anyhow::format_err!("mismatched message id"));
        }

        let m_operation = operations::from(&mut self.stream)?;
        if m_operation != operation {
            return Err(anyhow::format_err!("mismatched operation"));
        }

        let mut size_buff = 0usize.to_le_bytes();
        self.stream.read_exact(&mut size_buff)?;
        let size = usize::from_le_bytes(size_buff);

        let mut buff = vec![0u8; size];
        self.stream.read_exact(&mut buff)?;

        Ok(buff)
    }
}

impl<'a, T> Command<GetRequest<'a>, GetResponse<T>> for Client
where
    T: DeserializeOwned,
{
    fn send(&mut self, request: GetRequest<'a>) -> Result<GetResponse<T>> {
        let id = self.put_message(Operation::GET, &bincode::serialize(&request)?)?;
        let response_bytes = self.get_message(id, Operation::GET)?;

        let raw_response: GetResponse<Vec<u8>> = bincode::deserialize(&response_bytes)?;
        let value: T = bincode::deserialize(&raw_response.value)?;

        return Ok(GetResponse { value });
    }
}

impl<'a, T> Command<SetRequest<'a, T>, SetResponse> for Client
where
    T: Serialize,
{
    fn send(&mut self, request: SetRequest<'a, T>) -> Result<SetResponse> {
        let id = self.put_message(Operation::SET, &bincode::serialize(&request)?)?;
        let response_bytes = self.get_message(id, Operation::SET)?;

        // check that we got a valid response
        bincode::deserialize::<SetResponse>(&response_bytes)?;

        return Ok(SetResponse {});
    }
}
