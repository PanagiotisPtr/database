use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct GetRequest<'a> {
    pub key: &'a str,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetResponse<T> {
    pub value: T,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SetRequest<'a, T> {
    pub key: &'a str,
    pub value: T,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SetResponse {}
