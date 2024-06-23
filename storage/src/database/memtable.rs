use std::collections::BTreeMap;

use anyhow::Result;
use bytes::Bytes;
use commons::{
    command::Command,
    messages::{
        DelRequest, DelResponse, GetRequest, GetResponse, KeyType, SetRequest, SetResponse,
    },
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Memtable {
    data: BTreeMap<KeyType, Option<Bytes>>,
}

impl Memtable {
    pub fn get(&self, key: &KeyType) -> Option<Bytes> {
        self.data.get(key)?.clone()
    }

    pub fn set(&mut self, key: KeyType, value: Option<Bytes>) -> Result<()> {
        self.data.insert(key, value);
        Ok(())
    }

    pub fn get_data(&self) -> &BTreeMap<KeyType, Option<Bytes>> {
        &self.data
    }

    pub fn new() -> Self {
        Memtable {
            data: BTreeMap::new(),
        }
    }
}

impl Command<GetRequest, GetResponse<Bytes>> for Memtable {
    fn send(&mut self, request: GetRequest) -> Result<GetResponse<Bytes>> {
        Ok(GetResponse {
            value: self.get(&request.key),
        })
    }
}

impl Command<SetRequest<Bytes>, SetResponse> for Memtable {
    fn send(&mut self, request: SetRequest<Bytes>) -> Result<SetResponse> {
        self.set(request.key, Some(request.value))?;

        return Ok(SetResponse {});
    }
}

impl Command<DelRequest, DelResponse> for Memtable {
    fn send(&mut self, request: DelRequest) -> Result<DelResponse> {
        self.set(request.key, None)?;

        return Ok(DelResponse {});
    }
}
