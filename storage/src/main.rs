use anyhow::Result;
use bytes::Bytes;
use commons::spec::{KeyType, ReadStore, WriteStore};
use config::{Config as ConfigLoader, File, FileFormat};
use lsm_tree::LSMTree;

use crate::lsm_tree::config::{Config, LSMTreeConfig, SSTableConfig};

mod lsm_tree;
mod server;

fn main() -> Result<()> {
    let builder = ConfigLoader::builder()
        .set_default("max_memtable_size_bytes", 200)?
        .set_default("max_number_of_memtables", 4)?
        .set_default("sstable_block_size_bytes", 200)?
        .set_default("data_dir", "/var/database/data")?
        .add_source(File::new("config.toml", FileFormat::Toml));
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
    println!("config: {:?}", config);

    let mut lsm_tree = LSMTree::new(config);
    let to_bytes = |s: &str| -> Option<Bytes> { Some(Bytes::from(bincode::serialize(s).unwrap())) };
    let to_key = |s: &str| -> KeyType { KeyType::Str(s.to_string()) };
    lsm_tree.set(to_key("c0"), to_bytes("world")).unwrap();
    lsm_tree.set(to_key("e0"), to_bytes("world")).unwrap();
    lsm_tree.set(to_key("f0"), to_bytes("world")).unwrap();
    lsm_tree.set(to_key("l0"), to_bytes("world")).unwrap();
    lsm_tree.set(to_key("m0"), to_bytes("world")).unwrap();
    lsm_tree.set(to_key("n0"), to_bytes("world")).unwrap();
    lsm_tree.set(to_key("o0"), to_bytes("world")).unwrap();
    lsm_tree.set(to_key("g0"), to_bytes("world")).unwrap();
    lsm_tree.set(to_key("h0"), to_bytes("world")).unwrap();
    lsm_tree.set(to_key("i0"), to_bytes("world")).unwrap();
    lsm_tree.set(to_key("j0"), to_bytes("world")).unwrap();
    lsm_tree.set(to_key("b0"), to_bytes("world")).unwrap();
    lsm_tree.set(to_key("k0"), to_bytes("world")).unwrap();
    lsm_tree.set(to_key("p0"), to_bytes("world")).unwrap();
    lsm_tree.set(to_key("a0"), to_bytes("world")).unwrap();
    lsm_tree.set(to_key("c1"), to_bytes("world")).unwrap();
    lsm_tree.set(to_key("e1"), to_bytes("world")).unwrap();
    lsm_tree.set(to_key("f1"), to_bytes("world")).unwrap();
    lsm_tree.set(to_key("l1"), to_bytes("world")).unwrap();
    lsm_tree.set(to_key("m1"), to_bytes("world")).unwrap();
    lsm_tree.set(to_key("n1"), to_bytes("world")).unwrap();
    lsm_tree.set(to_key("o1"), to_bytes("world")).unwrap();
    lsm_tree.set(to_key("g1"), to_bytes("world")).unwrap();
    lsm_tree.set(to_key("h1"), to_bytes("world")).unwrap();
    lsm_tree.set(to_key("i1"), to_bytes("world")).unwrap();
    lsm_tree.set(to_key("j1"), to_bytes("world")).unwrap();
    lsm_tree.set(to_key("b1"), to_bytes("world")).unwrap();
    lsm_tree.set(to_key("k1"), to_bytes("world")).unwrap();
    lsm_tree.set(to_key("p1"), to_bytes("world")).unwrap();
    lsm_tree.set(to_key("a1"), to_bytes("world")).unwrap();

    println!("iterating from 'e'");
    for (key, value) in lsm_tree.scan(Some(to_key("e"))).unwrap() {
        println!("key: {:?}\t value: {:?}", key, value);
    }

    println!("getting: {:?}", to_key("c0"));
    let mut res = lsm_tree.get(to_key("c0"));
    println!("res: {:?}", res);

    println!("getting: {:?}", to_key("i0"));
    res = lsm_tree.get(to_key("i0"));
    println!("res: {:?}", res);

    println!("getting: {:?}", to_key("a0"));
    res = lsm_tree.get(to_key("a0"));
    println!("res: {:?}", res);

    println!("getting: {:?}", to_key("e0"));
    res = lsm_tree.get(to_key("e0"));
    println!("res: {:?}", res);

    println!("getting: {:?}", to_key("e1"));
    res = lsm_tree.get(to_key("a0"));
    println!("res: {:?}", res);

    println!("getting: {:?}", to_key("aaa"));
    res = lsm_tree.get(to_key("aaa"));
    println!("res: {:?}", res);
    println!("SSTable test");

    Ok(())
}
