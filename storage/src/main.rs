use rand::seq::SliceRandom;
use std::io::BufRead;
use std::time::Instant;
use std::{fs::File, io::BufReader};

use anyhow::Result;
use bytes::Bytes;
use commons::spec::{Entry, KeyType, ReadStore, WriteStore};
use config::{Config as ConfigLoader, File as ConfigFile, FileFormat};
use lsm_tree::LSMTree;

use crate::lsm_tree::config::{Config, LSMTreeConfig, SSTableConfig};

mod lsm_tree;
mod server;

fn main() -> Result<()> {
    let builder = ConfigLoader::builder()
        .set_default("max_memtable_size_bytes", 32768)?
        .set_default("max_number_of_memtables", 4)?
        .set_default("sstable_block_size_bytes", 4096)?
        .set_default("data_dir", "/var/database/data")?
        .add_source(ConfigFile::new("config.toml", FileFormat::Toml));
    let cfg = builder.build()?;
    let config = Config {
        lsm_tree_config: LSMTreeConfig {
            max_memtable_size_bytes: u64::try_from(cfg.get_int("max_memtable_size_bytes")?)?,
            max_number_of_memtables: usize::try_from(cfg.get_int("max_number_of_memtables")?)?,
            data_dir: cfg.get_string("data_dir")?,
        },
        sstable_config: SSTableConfig {
            sstable_block_size_bytes: u64::try_from(cfg.get_int("sstable_block_size_bytes")?)?,
        },
    };
    let mut entry_count = 0;
    let batch_size_bytes = 1 << 12;
    let mut entries: Vec<Entry> = vec![];
    println!("config: {:?}", config);
    println!("{:?} {:?}", entry_count, batch_size_bytes);

    let to_bytes = |s: &str| -> Option<Bytes> { Some(Bytes::from(bincode::serialize(s).unwrap())) };
    let to_key = |s: &str| -> KeyType { KeyType::Str(s.to_string()) };

    let file = File::open("/usr/share/dict/words")?;
    let reader = BufReader::new(file);
    let words: Vec<String> = reader.lines().collect::<Result<_, _>>()?;

    loop {
        let mut key = words.choose(&mut rand::thread_rng()).unwrap().clone();
        key.push_str(&words.choose(&mut rand::thread_rng()).unwrap().clone());
        let mut value = String::from("");
        for _ in 0..10 {
            let word = words.choose(&mut rand::thread_rng()).unwrap().clone();
            value.push_str(&word);
            value.push_str(" ");
        }
        entries.push((to_key(&key), to_bytes(&value)));
        let total_size = bincode::serialized_size(&entries)?;
        entry_count += 1;
        if total_size > batch_size_bytes {
            break;
        }
    }
    let entries_keys: Vec<KeyType> = entries.iter().map(|(k, _)| k.clone()).collect();

    let mut lsm_tree = LSMTree::new(config).unwrap();
    println!("Total entries: {}", entry_count);
    let now = Instant::now();

    for (key, value) in entries {
        lsm_tree.set(key, value)?;
    }

    let elapsed = now.elapsed();
    println!("Inserting time: {:.4?}", elapsed);

    /*
    let mut count = 0;
    let now = Instant::now();
    for _ in 0..100 {
        for (_, _) in lsm_tree.scan(None).unwrap() {
            count += 1;
        }
    }
    let elapsed = now.elapsed();
    println!("100 scans time: {:.4?}", elapsed);
    */

    let mut count = 0;
    for (_, _) in lsm_tree.scan(None).unwrap() {
        count += 1;
    }
    println!("total items: {:?}", count);

    /*
    let now = Instant::now();
    for key in entries_keys {
        lsm_tree.del(key)?;
    }
    let elapsed = now.elapsed();
    println!("Deleting time: {:.4?}", elapsed);
    */

    Ok(())
}
