use serde::{Deserialize, Serialize};

#[derive(Ord, Eq, PartialEq, PartialOrd, Serialize, Deserialize, Debug)]
pub enum KeyType {
    Int(i64),
    Str(String),
    Unsigned(u64),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetRequest {
    pub key: KeyType,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetResponse<T>
where
    T: Serialize + Clone,
{
    pub value: Option<T>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SetRequest<V>
where
    V: Serialize + Clone,
{
    pub key: KeyType,
    pub value: V,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SetResponse {}
