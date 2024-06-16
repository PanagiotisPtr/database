use std::{
    collections::BTreeMap,
    env,
    error::Error,
    fmt,
    fs::{read_dir, DirEntry, File, OpenOptions},
    io::{Read, Write},
    mem,
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
struct Segment<V>
where
    V: Serialize,
{
    data: BTreeMap<KeyType, V>,
}

impl<V> Segment<V>
where
    V: Serialize,
{
    fn get(&self, key: &KeyType) -> Option<&V> {
        self.data.get(key)
    }

    fn set(&mut self, key: KeyType, value: V) {
        self.data.insert(key, value);
    }

    pub fn new() -> Self {
        return Segment {
            data: BTreeMap::new(),
        };
    }
}

#[derive(Debug)]
struct InvalidReadError;

impl Error for InvalidReadError {}

impl fmt::Display for InvalidReadError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "failed to read expected amount of bytes")
    }
}

pub struct Memtable<V>
where
    V: Serialize,
{
    segments: Vec<Segment<V>>,
    log: File,
}

impl<V> Memtable<V>
where
    V: Serialize,
{
    fn commit<M>(&mut self, operation: Operation, message: M) -> Result<()>
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

    fn get_last_segment(&mut self) -> &mut Segment<V> {
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

    fn get(&self, key: &KeyType) -> Option<&V> {
        for segment in self.segments.iter().rev() {
            if let Some(v) = segment.get(key) {
                return Some(v);
            }
        }
        None
    }

    fn set(&mut self, key: K, value: V) -> Result<(), Box<dyn Error>> {
        self.get_last_segment().set(key, value);
        if self.get_last_segment().data.len() >= 5 {
            self.segments.push(Segment {
                data: BTreeMap::new(),
            });
            self.persist_segment(self.segments.len() - 2)?;
        }
        Ok(())
    }

    pub fn del(&mut self, key: String) -> Result<(), Box<dyn Error>> {
        self.get_last_segment().set(key, "NULL".to_string());
        Ok(())
    }

    fn read_usize(&self, file: &mut File) -> Result<usize, Box<dyn Error>> {
        let size_bytes = mem::size_of::<usize>();
        let mut buffer = usize::from(0u8).to_le_bytes();
        match file.read(&mut buffer) {
            Ok(n) => {
                if n != size_bytes {
                    Err(Box::new(InvalidReadError))
                } else {
                    Ok(usize::from_le_bytes(buffer))
                }
            }
            Err(e) => Err(Box::new(e)), // Handle error
        }
    }

    fn read_value(&self, file: &mut File, size: usize) -> Result<String, Box<dyn Error>> {
        let mut buffer = vec![0; size];
        match file.read(&mut buffer) {
            Ok(n) => {
                if n != size {
                    Err(Box::new(InvalidReadError))
                } else {
                    Ok(String::from_utf8(buffer)?)
                }
            }
            Err(e) => Err(Box::new(e)), // Handle error
        }
    }

    fn load_segment_from_file(&self, dir: DirEntry) -> Result<Segment, Box<dyn Error>> {
        let mut file = OpenOptions::new().read(true).open(dir.path())?;
        let metadata = file.metadata()?;
        let file_size = metadata.len();
        let mut read_bytes: u64 = 0;
        let mut segment = Segment::new();
        let size_bytes = u64::try_from(mem::size_of::<usize>())?;
        loop {
            let mut size = self.read_usize(&mut file)?;
            let key = self.read_value(&mut file, size)?;
            read_bytes = read_bytes + u64::try_from(size)? + size_bytes;

            size = self.read_usize(&mut file)?;
            let value = self.read_value(&mut file, size)?;
            read_bytes = read_bytes + u64::try_from(size)? + size_bytes;

            if read_bytes == file_size {
                break;
            }

            segment.data.insert(key, value);
        }

        Ok(segment)
    }

    fn load_segments(&self) -> Result<Vec<Segment>, Box<dyn Error>> {
        let data_location = match env::var(DATA_LOCATION_ENV_VAR) {
            Ok(v) => v,
            Err(_) => String::from(DATA_LOCATION_DEFAULT),
        };
        let data_dir = Path::new(&data_location);
        let paths = read_dir(data_dir)?;

        let mut segments: Vec<Segment> = vec![];
        let mut files = BTreeMap::new();
        for path in paths {
            let dir_entry = match path {
                Ok(v) => v,
                Err(_) => continue,
            };
            let file_type = match dir_entry.file_type() {
                Ok(v) => v,
                Err(_) => continue,
            };
            if !file_type.is_file() {
                continue;
            }
            let filename = match dir_entry.file_name().to_str() {
                Some(s) => s.to_string(),
                None => continue,
            };
            let parts: Vec<&str> = filename.split('.').collect();
            if parts.len() != 2 {
                continue;
            }
            let timestamp = match parts.get(0).unwrap().parse::<u64>() {
                Ok(v) => v,
                Err(_) => continue,
            };
            files.insert(timestamp, dir_entry);
        }

        for (_, dir_entry) in files {
            segments.push(self.load_segment_from_file(dir_entry)?);
        }

        Ok(segments)
    }

    pub fn new() -> Result<Self, Box<dyn Error>> {
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

        let mut table = Memtable {
            segments: vec![],
            log: file,
        };
        table.segments = table.load_segments()?;

        Ok(table)
    }
}

impl<V> Command<GetRequest, GetResponse<V>> for Memtable<V>
where
    V: Serialize + DeserializeOwned + Clone,
{
    fn send(&mut self, request: GetRequest) -> Result<GetResponse<V>> {
        return Ok(GetResponse {
            value: self.get(&request.key).cloned(),
        });
    }
}

impl<V> Command<SetRequest<V>, SetResponse> for Memtable<V>
where
    V: Serialize + DeserializeOwned + Clone,
{
    fn send(&mut self, request: SetRequest<V>) -> Result<SetResponse> {
        self.commit(Operation::SET, request)?;
        self.set(request.key, request.value)?;

        return Ok(SetResponse {});
    }
}
