use std::{
    collections::BTreeMap,
    env,
    fs::{File, OpenOptions},
    io::Write,
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::Result;
use commons::{
    command::Command,
    messages::{GetRequest, GetResponse, KeyType, SetRequest, SetResponse},
    operations::Operation,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

const LOG_LOCATION_ENV_VAR: &str = "LOG_DIR";
const LOG_LOCATION_DEFAULT: &str = "./logs";

const DATA_LOCATION_ENV_VAR: &str = "DATA_DIR";
const DATA_LOCATION_DEFAULT: &str = "./data";

#[derive(Serialize, Deserialize, Debug)]
struct Log {
    operation: Operation,
    message: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Segment {
    data: BTreeMap<KeyType, Vec<u8>>,
}

impl Segment {
    fn get(&self, key: &KeyType) -> Option<&Vec<u8>> {
        self.data.get(key)
    }

    fn set(&mut self, key: KeyType, value: Vec<u8>) {
        self.data.insert(key, value);
    }
}

pub struct Memtable {
    segments: Vec<Segment>,
    log: File,
}

impl Memtable {
    fn commit<M>(&mut self, operation: Operation, message: &M) -> Result<()>
    where
        M: Serialize,
    {
        bincode::serialize_into(
            &mut self.log,
            &Log {
                operation,
                message: bincode::serialize(&message)?,
            },
        )?;
        Ok(())
    }

    fn get_last_segment(&mut self) -> &mut Segment {
        if self.segments.len() == 0 {
            self.segments.push(Segment {
                data: BTreeMap::new(),
            });
        }
        self.segments.last_mut().unwrap()
    }

    fn get_segment_file(&self) -> Result<File> {
        let data_location = match env::var(DATA_LOCATION_ENV_VAR) {
            Ok(v) => v,
            Err(_) => String::from(DATA_LOCATION_DEFAULT),
        };
        let data_dir = Path::new(&data_location);
        let now = SystemTime::now();
        let timestamp = now.duration_since(UNIX_EPOCH).unwrap();
        let filename = format!("{}.data", timestamp.as_millis());
        let data_path = data_dir.join(filename);
        let file = OpenOptions::new()
            .read(true)
            .create(true)
            .write(true)
            .append(true)
            .open(data_path)?;

        Ok(file)
    }

    fn persist_segment(&self, segment_id: usize) -> Result<()> {
        let segment = self.segments.get(segment_id).unwrap();
        let mut file = self.get_segment_file()?;
        bincode::serialize_into(&mut file, &segment)?;
        file.flush()?;

        Ok(())
    }

    fn get(&self, key: &KeyType) -> Option<&Vec<u8>> {
        for segment in self.segments.iter().rev() {
            if let Some(v) = segment.get(key) {
                return Some(v);
            }
        }
        None
    }

    fn set(&mut self, key: KeyType, value: Vec<u8>) -> Result<()> {
        self.get_last_segment().set(key, value);
        if self.get_last_segment().data.len() >= 5 {
            self.segments.push(Segment {
                data: BTreeMap::new(),
            });
            self.persist_segment(self.segments.len() - 2)?;
        }
        Ok(())
    }

    pub fn new() -> Result<Self> {
        let log_location = match env::var(LOG_LOCATION_ENV_VAR) {
            Ok(v) => v,
            Err(_) => String::from(LOG_LOCATION_DEFAULT),
        };
        let log_dir = Path::new(&log_location);
        let now = SystemTime::now();
        let timestamp = now.duration_since(UNIX_EPOCH).unwrap();
        let filename = format!("{}.log", timestamp.as_millis());
        let log_path = log_dir.join(filename);
        let file = OpenOptions::new()
            .read(true)
            .create(true)
            .write(true)
            .append(true)
            .open(log_path)?;

        let table = Memtable {
            segments: vec![],
            log: file,
        };

        Ok(table)
    }
}

impl Command<GetRequest, GetResponse<Vec<u8>>> for Memtable {
    fn send(&mut self, request: GetRequest) -> Result<GetResponse<Vec<u8>>> {
        Ok(GetResponse {
            value: self.get(&request.key).cloned(),
        })
    }
}

impl Command<SetRequest<Vec<u8>>, SetResponse> for Memtable {
    fn send(&mut self, request: SetRequest<Vec<u8>>) -> Result<SetResponse> {
        self.commit(Operation::SET, &request)?;
        self.set(request.key, request.value)?;

        return Ok(SetResponse {});
    }
}
