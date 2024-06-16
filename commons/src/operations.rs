use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum Operation {
    PING,
    EXIT,
    GET,
    SET,
    DEL,
}
