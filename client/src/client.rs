use anyhow::Result;
use commons::{
    command::Command,
    messages::{GetRequest, GetResponse, SetRequest, SetResponse},
    operations::Operation,
    transport::{Message, MessageHeaders},
    versions::Version,
};
use rand::{rngs::ThreadRng, Rng};
use serde::{de::DeserializeOwned, Serialize};
use std::net::TcpStream;

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

    fn put_message(&mut self, operation: Operation, data: &[u8]) -> Result<u32> {
        let message = Message {
            headers: MessageHeaders {
                version: self.version.clone(),
                id: self.rng.gen::<u32>(),
                operation,
            },
            content: data.to_vec(),
        };

        bincode::serialize_into(&mut self.stream, &message)?;

        Ok(message.headers.id)
    }

    fn get_message(&mut self, id: u32, operation: Operation) -> Result<Vec<u8>> {
        let message: Message = bincode::deserialize_from(&mut self.stream)?;
        if message.headers.version != self.version {
            return Err(anyhow::format_err!("mismatched versions"));
        }

        if message.headers.id != id {
            return Err(anyhow::format_err!("mismatched message id"));
        }

        if message.headers.operation != operation {
            return Err(anyhow::format_err!("mismatched operation"));
        }

        Ok(message.content)
    }
}

impl<V> Command<GetRequest, GetResponse<V>> for Client
where
    V: Serialize + DeserializeOwned,
{
    fn send(&mut self, request: GetRequest) -> Result<GetResponse<V>> {
        let id = self.put_message(Operation::GET, &bincode::serialize(&request)?)?;
        let response_bytes = self.get_message(id, Operation::GET)?;

        let raw_response: GetResponse<Vec<u8>> = bincode::deserialize(&response_bytes)?;
        match raw_response.value {
            Some(v) => Ok(GetResponse {
                value: bincode::deserialize(&v)?,
            }),
            None => Ok(GetResponse { value: None }),
        }
    }
}

impl<V> Command<SetRequest<V>, SetResponse> for Client
where
    V: Serialize + Clone,
{
    fn send(&mut self, request: SetRequest<V>) -> Result<SetResponse> {
        let id = self.put_message(Operation::SET, &bincode::serialize(&request)?)?;
        let response_bytes = self.get_message(id, Operation::SET)?;

        // check that we got a valid response
        bincode::deserialize::<SetResponse>(&response_bytes)?;

        return Ok(SetResponse {});
    }
}
