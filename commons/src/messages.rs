use serde::{Deserialize, Serialize};

#[derive(Ord, Eq, PartialEq, PartialOrd, Serialize, Deserialize, Debug, Clone)]
pub enum KeyType {
    Int(i64),
    Str(String),
    Unsigned(u64),
}

#[derive(Eq, PartialEq, Serialize, Deserialize, Debug)]
pub struct GetRequest {
    pub key: KeyType,
}

#[derive(Eq, PartialEq, Serialize, Deserialize, Debug)]
pub struct GetResponse<T>
where
    T: Serialize + Clone,
{
    pub value: Option<T>,
}

#[derive(Eq, PartialEq, Serialize, Deserialize, Debug)]
pub struct SetRequest<V>
where
    V: Serialize + Clone,
{
    pub key: KeyType,
    pub value: V,
}

#[derive(Eq, PartialEq, Serialize, Deserialize, Debug)]
pub struct SetResponse {}

#[derive(Eq, PartialEq, Serialize, Deserialize, Debug)]
pub struct DelRequest {
    pub key: KeyType,
}

#[derive(Eq, PartialEq, Serialize, Deserialize, Debug)]
pub struct DelResponse {}

#[derive(Eq, PartialEq, Serialize, Deserialize, Debug)]
pub struct PingRequest {}

#[derive(Eq, PartialEq, Serialize, Deserialize, Debug)]
pub struct PingResponse {}

#[derive(Eq, PartialEq, Serialize, Deserialize, Debug)]
pub struct ExitRequest {}

#[derive(Eq, PartialEq, Serialize, Deserialize, Debug)]
pub struct ExitResponse {}
