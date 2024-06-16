use std::net::{TcpListener, TcpStream};

use anyhow::Result;
use commons::{
    messages::{GetRequest, GetResponse, KeyType, SetRequest, SetResponse},
    operations::Operation,
    transport::Message,
    versions::Version,
};

use crate::database::memtable::Memtable;
use commons::command::Command;

pub struct Server {
    version: Version,
    storage: Memtable,
}

impl Server {
    pub fn new() -> Self {
        Server {
            version: Version::V0,
            storage: Memtable::new().unwrap(),
        }
    }

    pub fn listen(&mut self, port: &str) {
        let listener = TcpListener::bind(format!("127.0.0.1:{}", port)).unwrap();
        for stream in listener.incoming() {
            let stream = stream.unwrap();

            self.handle_connection(stream).unwrap();
        }
    }

    fn handle_get(&mut self, content: Vec<u8>) -> Result<GetResponse<Vec<u8>>> {
        let request: GetRequest = bincode::deserialize(&content)?;
        self.storage.send(request)
    }

    fn handle_set(&mut self, content: Vec<u8>) -> Result<SetResponse> {
        let request: SetRequest<Vec<u8>> = bincode::deserialize(&content)?;
        self.storage.send(request)
    }

    fn handle_connection(&mut self, mut stream: TcpStream) -> Result<()> {
        loop {
            let message: Message = bincode::deserialize_from(&mut stream)?;
            if message.headers.version != self.version {
                continue;
            }
            match message.headers.operation {
                Operation::GET => {
                    let result = self.handle_get(message.content)?;
                    bincode::serialize_into(
                        &mut stream,
                        &Message {
                            headers: message.headers,
                            content: bincode::serialize(&result)?,
                        },
                    )?;
                }
                Operation::SET => {
                    let result = self.handle_set(message.content)?;
                    bincode::serialize_into(
                        &mut stream,
                        &Message {
                            headers: message.headers,
                            content: bincode::serialize(&result)?,
                        },
                    )?;
                }
                Operation::EXIT => break,
                _ => continue,
            };
        }

        Ok(())
    }
}
