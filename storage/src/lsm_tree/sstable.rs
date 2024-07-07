use std::{
    collections::BTreeMap,
    fs::File,
    io::{BufRead, Cursor, Read, Seek, SeekFrom},
    path::Path,
    sync::Arc,
};

use anyhow::Result;
use commons::spec::{Entry, EntryIterator, KeyType, ReadStore, ValueType};
use serde::{Deserialize, Serialize};
use uuid::{NoContext, Timestamp, Uuid};

use super::config::SSTableConfig;

#[derive(Debug, PartialEq, Serialize, Deserialize, Default)]
struct FilePointer {
    start: u64,
    size: u64,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Default)]
struct Header {
    index_ptr: FilePointer,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Block {
    file_ptr: FilePointer,
    num_entries: u64,

    #[serde(skip)]
    config: Arc<SSTableConfig>,
}

struct BlockIterator {
    idx: u64,
    total: u64,
    reader: Box<dyn BufRead>,
}

impl Block {
    fn new(config: Arc<SSTableConfig>) -> Self {
        Self {
            file_ptr: FilePointer::default(),
            num_entries: u64::default(),
            config,
        }
    }

    fn get(&self, file: &mut File, key: KeyType) -> Result<ValueType> {
        let iter = self.iter(file)?;
        for (k, v) in iter {
            if key.eq(&k) {
                return Ok(v);
            }
        }

        Ok(None)
    }

    fn scan(&self, key: Option<KeyType>, file: &mut File) -> Result<EntryIterator> {
        if let None = key {
            return self.iter(file);
        }
        let key = key.unwrap();
        Ok(Box::new(
            self.iter(file)?.skip_while(move |(k, _)| k.lt(&key)),
        ))
    }

    fn iter(&self, file: &mut File) -> Result<EntryIterator> {
        file.seek(SeekFrom::Start(self.file_ptr.start))?;
        let mut buffer = vec![0u8; self.config.sstable_block_size_bytes.try_into()?];
        file.read_exact(&mut buffer).unwrap();

        Ok(Box::new(BlockIterator {
            idx: 0,
            total: self.num_entries,
            reader: Box::new(Cursor::new(buffer)),
        }))
    }
}

impl Iterator for BlockIterator {
    type Item = (KeyType, ValueType);

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
    config: Arc<SSTableConfig>,
    header: Header,
    index: BTreeMap<KeyType, Block>,
    file: File,

    #[cfg(test)]
    pub filename: String,
}

pub struct SSTableIterator<'a> {
    key: Option<KeyType>,
    outer: Box<dyn Iterator<Item = &'a Block> + 'a>,
    inner: Option<EntryIterator<'a>>,
    file: &'a mut File,
}

impl<'a> Iterator for SSTableIterator<'a> {
    type Item = Entry;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(inner) = self.inner.as_mut() {
                if let Some(item) = inner.next() {
                    return Some(item);
                }
            }

            match self.outer.next() {
                Some(block) => {
                    let iter = block.scan(self.key.clone(), &mut self.file).ok()?;
                    self.inner = Some(iter);
                }
                None => return None,
            }
        }
    }
}

impl SSTable {
    pub fn new(config: Arc<SSTableConfig>, entries: EntryIterator) -> Result<Self> {
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
        let mut block = Block::new(Arc::clone(&config));
        let mut last_key: Option<KeyType> = None;
        for (key, value) in entries {
            let entry_size = bincode::serialized_size(&key)? + bincode::serialized_size(&value)?;
            if block.num_entries > 0
                && entry_size + block.file_ptr.size > config.sstable_block_size_bytes
            {
                index.insert(last_key.unwrap().clone(), block);
                block = Block::new(Arc::clone(&config));
                block.file_ptr.start = start;
            }
            last_key = Some(key.clone());
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
            config: Arc::clone(&config),
            header,
            index,
            file: File::open(&file_loc)?,

            #[cfg(test)]
            filename,
        })
    }

    pub fn load(config: Arc<SSTableConfig>, path: &Path) -> Result<Self> {
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
            config: Arc::clone(&config),
            header,
            index,
            file,
            #[cfg(test)]
            filename,
        })
    }
}

impl<'a> ReadStore<'a> for SSTable {
    fn get(&'a mut self, key: KeyType) -> Result<ValueType> {
        let mut range = self.index.range(key.clone()..).peekable();
        match range.peek() {
            None => Ok(None),
            Some((_, b)) => b.get(&mut self.file, key),
        }
    }

    fn scan(&'a mut self, key: Option<KeyType>) -> Result<EntryIterator> {
        let outer: Box<dyn Iterator<Item = &'a Block> + 'a> = match &key {
            Some(key) => Box::new(self.index.range(key.clone()..).map(|(_, b)| b)),
            None => Box::new(self.index.values()),
        };

        Ok(Box::new(
            SSTableIterator {
                key: key.clone(),
                outer,
                inner: None,
                file: &mut self.file,
            }
            .into_iter(),
        ))
    }
}
