use anyhow::Result;
use bytes::Bytes;
use serde::{Deserialize, Serialize};

#[derive(Ord, Eq, PartialEq, PartialOrd, Serialize, Deserialize, Debug, Clone)]
pub enum KeyType {
    Int(i64),
    Str(String),
    Unsigned(u64),
}

pub type ValueType = Option<Bytes>;
pub type Entry = (KeyType, ValueType);
pub type EntryIterator<'a> = Box<dyn Iterator<Item = Entry> + 'a>;

pub trait ReadStore<'a> {
    fn get(&'a mut self, key: KeyType) -> Result<ValueType>;

    fn scan(&'a mut self, key: Option<KeyType>) -> Result<EntryIterator<'a>>;
}

pub trait WriteStore<'a> {
    fn set(&'a mut self, key: KeyType, val: ValueType) -> Result<()>;

    fn del(&'a mut self, key: KeyType) -> Result<()> {
        self.set(key, None)
    }
}
