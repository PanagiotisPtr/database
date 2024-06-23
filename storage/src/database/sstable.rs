use std::{
    collections::BTreeMap,
    fs::File,
    io::{Read, Seek, SeekFrom},
    path::Path,
};

use crate::database::memtable::Memtable;
use anyhow::Result;
use commons::messages::KeyType;
use serde::{Deserialize, Serialize};
use uuid::{NoContext, Timestamp, Uuid};

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
struct SSTable {
    header: Header,
    index: BTreeMap<KeyType, FilePointer>,

    #[cfg(test)]
    pub filename: String,
}

impl SSTable {
    pub fn new(memtable: &Memtable) -> Result<Self> {
        let data_dir = "./data";
        let path = std::path::Path::new(data_dir);
        if !path.exists() {
            std::fs::create_dir_all(data_dir)?;
        }
        let ts = Timestamp::now(NoContext);
        let filename = Uuid::new_v7(ts).to_string() + ".sst";
        let mut file = File::create(path.join(filename.clone()))?;

        let mut index = BTreeMap::new();
        let mut start = 0;
        let mut end;
        for (key, value) in memtable.get_data() {
            bincode::serialize_into(&mut file, &key)?;
            bincode::serialize_into(&mut file, &value)?;
            end = file.seek(SeekFrom::Current(0))?;
            index.insert(
                key.clone(),
                FilePointer {
                    start,
                    size: end - start,
                },
            );
            start = end;
        }
        println!("writing index at: {}", start);
        bincode::serialize_into(&mut file, &index)?;
        end = file.seek(SeekFrom::Current(0))?;
        println!("wrote index until: {}", end);
        println!("index size: {}", end - start);
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

            #[cfg(test)]
            filename,
        })
    }

    pub fn load(path: &Path) -> Result<Self> {
        let mut file = Box::new(File::open(path)?);
        let metadata = file.metadata()?;
        let header_size = bincode::serialized_size(&Header::default())?;

        file.seek(SeekFrom::Start(metadata.len() - header_size))?;
        let header: Header = bincode::deserialize_from(&mut file)?;
        file.seek(SeekFrom::Start(header.index_ptr.start))?;
        let mut buffer = vec![0u8; header.index_ptr.size.try_into()?];
        file.read_exact(&mut buffer)?;
        let index = bincode::deserialize(&mut buffer)?;

        #[cfg(test)]
        let filename = path
            .file_name()
            .and_then(|name| name.to_str())
            .map(|name| name.to_string())
            .unwrap_or("".to_string());

        Ok(SSTable {
            header,
            index,
            #[cfg(test)]
            filename,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::Bytes;

    #[test]
    fn constructor_simple_test() {
        let mut memtable = crate::database::memtable::Memtable::new();
        memtable
            .set(
                KeyType::Str(String::from("hello")),
                Some(Bytes::from(bincode::serialize("world").unwrap())),
            )
            .unwrap();
        memtable
            .set(
                KeyType::Str(String::from("ananas")),
                Some(Bytes::from(bincode::serialize("banana").unwrap())),
            )
            .unwrap();
        memtable
            .set(
                KeyType::Str(String::from("zero")),
                Some(Bytes::from(bincode::serialize("one").unwrap())),
            )
            .unwrap();

        let sstable = SSTable::new(&memtable).unwrap();
        let another_table = SSTable::load(&Path::new("./data").join(&sstable.filename)).unwrap();

        assert_eq!(sstable, another_table);
    }
}
