use std::{collections::BTreeMap, iter::Peekable};

use anyhow::Result;
use bytes::Bytes;
use commons::messages::KeyType;
use serde::{Deserialize, Serialize};

pub struct Active;
pub struct Locked;

pub type ValueType = Option<Bytes>;

#[derive(Serialize, Deserialize, Debug)]
pub struct Memtable<T = Active> {
    status: std::marker::PhantomData<T>,
    data: BTreeMap<KeyType, ValueType>,
}

impl<T> Memtable<T> {
    pub fn get(&self, key: &KeyType) -> Option<&ValueType> {
        self.data.get(key)
    }

    pub fn scan(
        &self,
        key: Option<&KeyType>,
    ) -> Peekable<Box<dyn Iterator<Item = (&KeyType, &Option<Bytes>)> + '_>> {
        let iter: Box<dyn Iterator<Item = _>> = match key {
            None => Box::new(self.data.iter()),
            Some(k) => Box::new(self.data.range(k..)),
        };
        iter.peekable()
    }
}

impl Memtable<Active> {
    pub fn new() -> Self {
        Memtable::<Active> {
            status: std::marker::PhantomData,
            data: BTreeMap::new(),
        }
    }

    pub fn set(&mut self, key: KeyType, value: Option<Bytes>) -> Result<()> {
        self.data.insert(key, value);
        Ok(())
    }

    pub fn del(&mut self, key: KeyType) -> Result<()> {
        self.set(key, None)
    }

    pub fn lock(self) -> Memtable<Locked> {
        self.into()
    }

    pub fn size_bytes(&self) -> Result<u64> {
        Ok(bincode::serialized_size(self)?)
    }
}

impl Default for Memtable<Active> {
    fn default() -> Self {
        Memtable::new()
    }
}

impl From<Memtable<Active>> for Memtable<Locked> {
    fn from(value: Memtable<Active>) -> Self {
        Memtable::<Locked> {
            status: std::marker::PhantomData,
            data: value.data,
        }
    }
}

impl Memtable<Locked> {
    pub fn get_data(&self) -> &BTreeMap<KeyType, Option<Bytes>> {
        &self.data
    }
}
