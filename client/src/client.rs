use anyhow::Result;
use bytes::Bytes;
use commons::{
    command::Command,
    messages::{
        DelRequest, DelResponse, ExitRequest, ExitResponse, GetRequest, GetResponse, PingRequest,
        PingResponse, SetRequest, SetResponse,
    },
    operations::Operation,
    transport::{Message, MessageHeaders},
    versions::Version,
};
use rand::{rngs::ThreadRng, Rng};
use serde::{de::DeserializeOwned, Serialize};
use std::{
    io,
    net::{Shutdown, TcpStream},
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

    fn put_message(&mut self, operation: Operation, data: Bytes) -> Result<u32> {
        let message = Message {
            headers: MessageHeaders {
                version: self.version.clone(),
                id: self.rng.gen::<u32>(),
                operation,
            },
            content: data,
        };

        bincode::serialize_into(&mut self.stream, &message)?;

        Ok(message.headers.id)
    }

    fn get_message<M>(&mut self, id: u32, operation: Operation) -> Result<M>
    where
        M: DeserializeOwned,
    {
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

        Ok(bincode::deserialize(&message.content)?)
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        match self.stream.shutdown(Shutdown::Both) {
            Ok(_) => return,
            Err(ref e) => {
                if e.kind() == io::ErrorKind::NotConnected {
                    return;
                } else {
                    eprintln!("failed to close TcpStream: {}", e)
                }
            }
        }
    }
}

impl<V> Command<GetRequest, GetResponse<V>> for Client
where
    V: Serialize + DeserializeOwned + Clone,
{
    fn send(&mut self, request: GetRequest) -> Result<GetResponse<V>> {
        let id = self.put_message(Operation::GET, bincode::serialize(&request)?.into())?;
        let raw_response: GetResponse<Bytes> = self.get_message(id, Operation::GET)?;

        match raw_response.value {
            Some(v) => Ok(GetResponse {
                value: Some(bincode::deserialize(&v)?),
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
        let id = self.put_message(
            Operation::SET,
            bincode::serialize(&SetRequest::<Bytes> {
                key: request.key,
                value: bincode::serialize(&request.value)?.into(),
            })?
            .into(),
        )?;
        self.get_message::<SetResponse>(id, Operation::SET)
    }
}

impl Command<DelRequest, DelResponse> for Client {
    fn send(&mut self, request: DelRequest) -> Result<DelResponse> {
        let id = self.put_message(
            Operation::DEL,
            bincode::serialize(&DelRequest { key: request.key })?.into(),
        )?;
        self.get_message::<DelResponse>(id, Operation::DEL)
    }
}

impl Command<PingRequest, PingResponse> for Client {
    fn send(&mut self, _: PingRequest) -> Result<PingResponse> {
        let id = self.put_message(Operation::PING, bincode::serialize(&PingRequest {})?.into())?;
        self.get_message::<PingResponse>(id, Operation::PING)
    }
}

impl Command<ExitRequest, ExitResponse> for Client {
    fn send(&mut self, _: ExitRequest) -> Result<ExitResponse> {
        let id = self.put_message(Operation::EXIT, bincode::serialize(&ExitRequest {})?.into())?;
        self.get_message::<ExitResponse>(id, Operation::EXIT)
    }
}
