use std::{
    io::prelude::*,
    net::{TcpListener, TcpStream},
};

use crate::database::memtable::Memtable;

pub struct Server {
    storage: Memtable,
}

impl Server {
    pub fn new() -> Self {
        Server {
            storage: Memtable::new().unwrap(),
        }
    }

    pub fn listen(&mut self, port: &str) {
        let listener = TcpListener::bind(format!("127.0.0.1:{}", port)).unwrap();
        for stream in listener.incoming() {
            let stream = stream.unwrap();

            self.handle_connection_old(stream);
        }
    }

    fn handle_connection_old(&mut self, mut stream: TcpStream) {
        loop {
            let buf = &mut [0u8];
            let mut data: Vec<u8> = vec![];
            loop {
                match stream.read_exact(buf) {
                    Ok(_) => {
                        if char::from(*buf.get(0).unwrap()) == '\n' {
                            break;
                        }
                        data.push(*buf.get(0).unwrap())
                    }
                    Err(e) => {
                        println!("error: {}", e);
                        break;
                    }
                }
            }
            if data.len() == 0 {
                break;
            }
            let line = String::from_utf8(data).unwrap();
            let parts: Vec<&str> = line.split(" ").collect();
            match parts.get(0).unwrap().to_uppercase().as_str() {
                "EXIT" => break,
                "PING" => stream.write_all("PONG".as_bytes()).unwrap(),
                "GET" => match self.storage.get(&String::from(*parts.get(1).unwrap())) {
                    Some(v) => stream.write_all(v.as_bytes()).unwrap(),
                    None => stream.write_all("NULL".as_bytes()).unwrap(),
                },
                "SET" => match self.storage.set(
                    String::from(*parts.get(1).unwrap()),
                    String::from(*parts.get(2).unwrap()),
                ) {
                    Ok(_) => stream.write_all("OK".as_bytes()).unwrap(),
                    Err(_) => stream.write_all("ERROR".as_bytes()).unwrap(),
                },
                "DEL" => match self.storage.del(String::from(*parts.get(1).unwrap())) {
                    Ok(_) => stream.write_all("OK".as_bytes()).unwrap(),
                    Err(_) => stream.write_all("ERROR".as_bytes()).unwrap(),
                },
                _ => stream.write_all("invalid input".as_bytes()).unwrap(),
            };
            stream.write_all("\n".as_bytes()).unwrap();
        }
    }
}
