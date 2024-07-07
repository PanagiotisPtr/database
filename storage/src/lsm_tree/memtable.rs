use std::collections::BTreeMap;

use anyhow::Result;
use commons::spec::{EntryIterator, KeyType, ReadStore, ValueType, WriteStore};
use serde::{Deserialize, Serialize};

pub struct Active;
pub struct Locked;

#[derive(Serialize, Deserialize, Debug)]
pub struct Memtable<T = Active> {
    status: std::marker::PhantomData<T>,
    data: BTreeMap<KeyType, ValueType>,
}

impl<'a, T> ReadStore<'a> for Memtable<T> {
    fn get(&'a mut self, key: KeyType) -> Result<ValueType> {
        Ok(match self.data.get(&key) {
            Some(v) => match v {
                None => None,
                Some(b) => Some(b.clone()),
            },
            None => None,
        })
    }

    fn scan(&'a mut self, key: Option<KeyType>) -> Result<EntryIterator<'a>> {
        let iter: EntryIterator<'a> = match key {
            None => Box::new(
                self.data
                    .iter()
                    .map(|(k, v)| (k.clone(), v.as_ref().map(|v| v.clone()))),
            ),
            Some(k) => Box::new(self.data.range(k..).map(|(k, v)| (k.clone(), v.clone()))),
        };
        Ok(iter)
    }
}

impl<'a, T> WriteStore<'a> for Memtable<T> {
    fn set(&'a mut self, key: KeyType, val: ValueType) -> Result<()> {
        self.data.insert(key, val);
        Ok(())
    }
}

impl Memtable<Active> {
    pub fn new() -> Self {
        Memtable::<Active> {
            status: std::marker::PhantomData,
            data: BTreeMap::new(),
        }
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
