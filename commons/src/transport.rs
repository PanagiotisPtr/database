use crate::{operations::Operation, versions::Version};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct MessageHeaders {
    pub version: Version,
    pub id: u32,
    pub operation: Operation,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Message {
    pub headers: MessageHeaders,
    pub content: Vec<u8>,
}
