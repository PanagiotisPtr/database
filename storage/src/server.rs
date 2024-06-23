use std::{
    io,
    net::{Shutdown, TcpListener, TcpStream},
};

use anyhow::Result;
use commons::{
    messages::{
        DelResponse, ExitRequest, ExitResponse, GetResponse, PingRequest, PingResponse, SetResponse,
    },
    operations::Operation,
    transport::Message,
    versions::Version,
};

use crate::database::memtable::Memtable;
use bytes::Bytes;
use commons::command::Command;

pub struct Server {
    version: Version,
    storage: Memtable,
}

impl Server {
    pub fn new() -> Self {
        Server {
            version: Version::V0,
            storage: Memtable::new(),
        }
    }

    pub fn listen(&mut self, port: &str) {
        let listener = TcpListener::bind(format!("127.0.0.1:{}", port)).unwrap();
        for stream in listener.incoming() {
            let stream = stream.unwrap();

            match self.handle_connection(stream) {
                Ok(_) => continue,
                Err(e) => {
                    eprintln!("failed to close TcpStream: {}", e)
                }
            }
        }
    }

    fn handle_get(&mut self, content: Bytes) -> Result<GetResponse<Bytes>> {
        let request = bincode::deserialize(&content)?;
        self.storage.send(request)
    }

    fn handle_set(&mut self, content: Bytes) -> Result<SetResponse> {
        let request = bincode::deserialize(&content)?;
        self.storage.send(request)
    }

    fn handle_del(&mut self, content: Bytes) -> Result<DelResponse> {
        let request = bincode::deserialize(&content)?;
        self.storage.send(request)
    }

    fn handle_ping(&mut self, content: Bytes) -> Result<PingResponse> {
        bincode::deserialize::<PingRequest>(&content)?;
        Ok(PingResponse {})
    }

    fn handle_exit(&mut self, content: Bytes) -> Result<ExitResponse> {
        bincode::deserialize::<ExitRequest>(&content)?;
        Ok(ExitResponse {})
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
                            content: bincode::serialize(&result)?.into(),
                        },
                    )?;
                }
                Operation::SET => {
                    let result = self.handle_set(message.content)?;
                    bincode::serialize_into(
                        &mut stream,
                        &Message {
                            headers: message.headers,
                            content: bincode::serialize(&result)?.into(),
                        },
                    )?;
                }
                Operation::DEL => {
                    let result = self.handle_del(message.content)?;
                    bincode::serialize_into(
                        &mut stream,
                        &Message {
                            headers: message.headers,
                            content: bincode::serialize(&result)?.into(),
                        },
                    )?;
                }
                Operation::PING => {
                    let result = self.handle_ping(message.content)?;
                    bincode::serialize_into(
                        &mut stream,
                        &Message {
                            headers: message.headers,
                            content: bincode::serialize(&result)?.into(),
                        },
                    )?;
                }
                Operation::EXIT => {
                    let result = self.handle_exit(message.content)?;
                    bincode::serialize_into(
                        &mut stream,
                        &Message {
                            headers: message.headers,
                            content: bincode::serialize(&result)?.into(),
                        },
                    )?;
                    break;
                }
            };
        }

        match stream.shutdown(Shutdown::Both) {
            Ok(_) => Ok(()),
            Err(ref e) => {
                if e.kind() == io::ErrorKind::NotConnected {
                    Ok(())
                } else {
                    Err(anyhow::format_err!("failed to close TcpStream: {}", e))
                }
            }
        }
    }
}
