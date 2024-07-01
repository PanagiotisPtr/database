use std::{
    collections::BTreeMap,
    fs::File,
    io::{BufRead, Cursor, Read, Seek, SeekFrom},
    iter::Peekable,
    path::Path,
    sync::{Arc, Mutex},
};

use crate::lsm_tree::memtable::Memtable;
use anyhow::Result;
use bytes::Bytes;
use commons::messages::KeyType;
use serde::{Deserialize, Serialize};
use uuid::{NoContext, Timestamp, Uuid};

use super::memtable::Locked;

const SSTABLE_BLOCK_SIZE: u64 = 200;

#[derive(Debug, PartialEq, Serialize, Deserialize, Default)]
struct FilePointer {
    start: u64,
    size: u64,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Default)]
struct Header {
    index_ptr: FilePointer,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Default)]
struct Block {
    file_ptr: FilePointer,
    num_entries: u64,
}

struct BlockIterator {
    idx: u64,
    total: u64,
    reader: Box<dyn BufRead>,
}

impl Block {
    fn get(&self, file: Arc<Mutex<File>>, key: &KeyType) -> Option<Bytes> {
        let iter = self.iter(file).ok()?;
        for (k, v) in iter {
            if key.eq(&k) {
                return v;
            }
        }

        None
    }

    fn scan(&self, key: &KeyType, file: Arc<Mutex<File>>) -> Result<Peekable<BlockIterator>> {
        let iter = self.iter(file)?;
        let mut peekable = iter.peekable();

        loop {
            match peekable.peek() {
                None => break,
                Some((k, _)) => {
                    if k.eq(key) {
                        break;
                    } else {
                        peekable.next();
                    }
                }
            };
        }

        Ok(peekable)
    }

    fn iter(&self, file: Arc<Mutex<File>>) -> Result<BlockIterator> {
        let mut file = file.lock().unwrap();
        file.seek(SeekFrom::Start(self.file_ptr.start))?;
        let mut buffer = vec![0u8; SSTABLE_BLOCK_SIZE.try_into()?];
        file.read_exact(&mut buffer).unwrap();

        Ok(BlockIterator {
            idx: 0,
            total: self.num_entries,
            reader: Box::new(Cursor::new(buffer)),
        })
    }
}

impl Iterator for BlockIterator {
    type Item = (KeyType, Option<Bytes>);

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx == self.total {
            None
        } else {
            let value: Self::Item = bincode::deserialize_from(&mut self.reader).ok()?;
            self.idx += 1;
            Some(value)
        }
    }
}

#[derive(Debug)]
pub struct SSTable {
    header: Header,
    index: BTreeMap<KeyType, Block>,
    file: Arc<Mutex<File>>,

    #[cfg(test)]
    pub filename: String,
}

pub struct SSTableIterator<'a> {
    outer: Peekable<std::collections::btree_map::Range<'a, KeyType, Block>>,
    inner: Option<Peekable<BlockIterator>>,
    file: Arc<Mutex<File>>,
}

impl<'a> Iterator for SSTableIterator<'a> {
    type Item = (KeyType, Option<Bytes>);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(inner) = self.inner.as_mut() {
                if let Some(item) = inner.next() {
                    return Some(item);
                }
            }

            match self.outer.next() {
                Some((_, block)) => {
                    let iter = block.iter(Arc::clone(&self.file)).ok()?;
                    self.inner = Some(iter.peekable());
                }
                None => return None,
            }
        }
    }
}

impl SSTable {
    pub fn new(memtable: &Memtable<Locked>) -> Result<Self> {
        let data_dir = "./data";
        let path = std::path::Path::new(data_dir);
        if !path.exists() {
            std::fs::create_dir_all(data_dir)?;
        }
        let ts = Timestamp::now(NoContext);
        let filename = Uuid::new_v7(ts).to_string() + ".sst";
        let file_loc = path.join(filename.clone());
        let mut file = File::create(&file_loc)?;

        let mut index: BTreeMap<KeyType, Block> = BTreeMap::new();
        let mut start = 0;
        let mut end;
        let mut block = Block::default();
        let mut last_key: Option<&KeyType> = None;
        for (key, value) in memtable.get_data() {
            let entry_size = bincode::serialized_size(&key)? + bincode::serialized_size(&value)?;
            if block.num_entries > 0 && entry_size + block.file_ptr.size > SSTABLE_BLOCK_SIZE {
                index.insert(last_key.unwrap().clone(), block);
                block = Block::default();
                block.file_ptr.start = start;
                last_key = None;
            }
            if last_key == None {
                last_key = Some(key);
            }
            bincode::serialize_into(&mut file, &key)?;
            bincode::serialize_into(&mut file, &value)?;
            end = file.seek(SeekFrom::Current(0))?;
            start = end;
            block.file_ptr.size = end - block.file_ptr.start;
            block.num_entries += 1;
        }
        if block.file_ptr.size > 0 {
            index.insert(last_key.unwrap().clone(), block);
        }
        bincode::serialize_into(&mut file, &index)?;
        end = file.seek(SeekFrom::Current(0))?;
        let header = Header {
            index_ptr: FilePointer {
                start,
                size: end - start,
            },
        };
        bincode::serialize_into(&mut file, &header)?;

        Ok(SSTable {
            header,
            index,
            file: Arc::new(Mutex::new(File::open(&file_loc)?)),

            #[cfg(test)]
            filename,
        })
    }

    pub fn get(&mut self, key: &KeyType) -> Option<Bytes> {
        let mut range = self.index.range(..=key).rev().peekable();
        match range.peek() {
            None => None,
            Some((_, b)) => b.get(Arc::clone(&self.file), key),
        }
    }

    pub fn scan(&mut self, key: &KeyType) -> SSTableIterator {
        let entry = self.index.range(..=key).rev().next();
        SSTableIterator {
            outer: self.index.range(key..).peekable(),
            inner: match entry {
                None => None,
                Some((_, b)) => b.scan(key, Arc::clone(&self.file)).ok(),
            },
            file: Arc::clone(&self.file),
        }
    }

    pub fn load(path: &Path) -> Result<Self> {
        let mut file = File::open(path)?;
        let metadata = file.metadata()?;
        let header_size = bincode::serialized_size(&Header::default())?;

        file.seek(SeekFrom::Start(metadata.len() - header_size))?;
        let header: Header = bincode::deserialize_from(&mut file)?;
        file.seek(SeekFrom::Start(header.index_ptr.start))?;
        let index = bincode::deserialize_from(&mut file)?;

        #[cfg(test)]
        let filename = path
            .file_name()
            .and_then(|name| name.to_str())
            .map(|name| name.to_string())
            .unwrap_or("".to_string());

        Ok(SSTable {
            header,
            index,
            file: Arc::new(Mutex::new(file)),
            #[cfg(test)]
            filename,
        })
    }
}
