use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Deserialize, Serialize, Default)]
pub struct Config {
    pub lsm_tree_config: LSMTreeConfig,
    pub sstable_config: SSTableConfig,
}

#[derive(Debug, PartialEq, Deserialize, Serialize, Default, Clone)]
pub struct SSTableConfig {
    pub sstable_block_size_bytes: u64,
}

#[derive(Debug, Deserialize, PartialEq, Serialize, Default, Clone)]
pub struct LSMTreeConfig {
    pub max_memtable_size_bytes: u64,
    pub max_number_of_memtables: usize,
    pub data_dir: String,
}

impl Config {
    pub fn new_default() -> Self {
        Config {
            lsm_tree_config: LSMTreeConfig {
                max_memtable_size_bytes: 1048576,
                max_number_of_memtables: 4,
                data_dir: "/var/database/data".to_owned(),
            },
            sstable_config: SSTableConfig {
                sstable_block_size_bytes: 200,
            },
        }
    }
}
