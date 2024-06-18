use std::{
    collections::BTreeMap,
    env,
    fs::{File, OpenOptions},
    io::Write,
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::Result;
use bytes::Bytes;
use commons::{
    command::Command,
    messages::{
        DelRequest, DelResponse, GetRequest, GetResponse, KeyType, SetRequest, SetResponse,
    },
    operations::Operation,
};
use serde::{Deserialize, Serialize};

const LOG_LOCATION_ENV_VAR: &str = "LOG_DIR";
const LOG_LOCATION_DEFAULT: &str = "./logs";

const DATA_LOCATION_ENV_VAR: &str = "DATA_DIR";
const DATA_LOCATION_DEFAULT: &str = "./data";

const MAX_PAGE_SIZE_BYTES: usize = 1_000_000; // 1MB

#[derive(Serialize, Deserialize, Debug)]
struct Log {
    operation: Operation,
    message: Bytes,
}

#[derive(Serialize, Deserialize, Debug)]
struct Segment {
    data: BTreeMap<KeyType, Option<Bytes>>,
}

impl Segment {
    fn get(&self, key: &KeyType) -> Option<Bytes> {
        match self.data.get(key) {
            Some(v) => match v {
                Some(vv) => Some(vv.clone()),
                None => None,
            },
            None => None,
        }
    }

    fn set(&mut self, key: KeyType, value: Option<Bytes>) {
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
                message: bincode::serialize(&message)?.into(),
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

    fn get(&self, key: &KeyType) -> Option<Bytes> {
        for segment in self.segments.iter().rev() {
            if let Some(v) = segment.get(key) {
                return Some(v);
            }
        }
        None
    }

    fn set(&mut self, key: KeyType, value: Option<Bytes>) -> Result<()> {
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

impl Command<GetRequest, GetResponse<Bytes>> for Memtable {
    fn send(&mut self, request: GetRequest) -> Result<GetResponse<Bytes>> {
        Ok(GetResponse {
            value: self.get(&request.key),
        })
    }
}

impl Command<SetRequest<Bytes>, SetResponse> for Memtable {
    fn send(&mut self, request: SetRequest<Bytes>) -> Result<SetResponse> {
        self.commit(Operation::SET, &request)?;
        self.set(request.key, Some(request.value))?;

        return Ok(SetResponse {});
    }
}

impl Command<DelRequest, DelResponse> for Memtable {
    fn send(&mut self, request: DelRequest) -> Result<DelResponse> {
        self.commit(Operation::DEL, &request)?;
        self.set(request.key, None)?;

        return Ok(DelResponse {});
    }
}
