use bytes::Bytes;
use commons::spec::{KeyType, ReadStore, WriteStore};
use lsm_tree::LSMTree;

use crate::lsm_tree::{memtable::Memtable, sstable::SSTable};

mod lsm_tree;
mod server;

fn main() {
    let mut lsm_tree = LSMTree::new();
    let to_bytes = |s: &str| -> Option<Bytes> { Some(Bytes::from(bincode::serialize(s).unwrap())) };
    let to_key = |s: &str| -> KeyType { KeyType::Str(s.to_string()) };
    lsm_tree.set(to_key("c"), to_bytes("world")).unwrap();
    lsm_tree.set(to_key("e"), to_bytes("world")).unwrap();
    lsm_tree.set(to_key("f"), to_bytes("world")).unwrap();
    lsm_tree.set(to_key("l"), to_bytes("world")).unwrap();
    lsm_tree.set(to_key("m"), to_bytes("world")).unwrap();
    lsm_tree.set(to_key("n"), to_bytes("world")).unwrap();
    lsm_tree.set(to_key("o"), to_bytes("world")).unwrap();
    lsm_tree.set(to_key("g"), to_bytes("world")).unwrap();
    lsm_tree.set(to_key("h"), to_bytes("world")).unwrap();
    lsm_tree.set(to_key("i"), to_bytes("world")).unwrap();
    lsm_tree.set(to_key("j"), to_bytes("world")).unwrap();
    lsm_tree.set(to_key("b"), to_bytes("world")).unwrap();
    lsm_tree.set(to_key("k"), to_bytes("world")).unwrap();
    lsm_tree.set(to_key("p"), to_bytes("world")).unwrap();
    lsm_tree.set(to_key("a"), to_bytes("world")).unwrap();

    println!("iterating from 'e'");
    let mut i = 0;
    for (key, value) in lsm_tree.scan(Some(to_key("e"))).unwrap() {
        println!("key: {:?}\t value: {:?}", key, value);
        if i > 15 {
            println!("oops!");
            break;
        }
        i += 1;
    }

    println!("SSTable test");

    let mut memtable = Memtable::new();
    memtable.set(to_key("c"), to_bytes("world")).unwrap();
    memtable.set(to_key("e"), to_bytes("world")).unwrap();
    memtable.set(to_key("f"), to_bytes("world")).unwrap();
    memtable.set(to_key("l"), to_bytes("world")).unwrap();
    memtable.set(to_key("m"), to_bytes("world")).unwrap();
    memtable.set(to_key("n"), to_bytes("world")).unwrap();
    memtable.set(to_key("o"), to_bytes("world")).unwrap();
    memtable.set(to_key("g"), to_bytes("world")).unwrap();
    memtable.set(to_key("h"), to_bytes("world")).unwrap();
    memtable.set(to_key("i"), to_bytes("world")).unwrap();
    memtable.set(to_key("j"), to_bytes("world")).unwrap();
    memtable.set(to_key("b"), to_bytes("world")).unwrap();
    memtable.set(to_key("k"), to_bytes("world")).unwrap();
    memtable.set(to_key("p"), to_bytes("world")).unwrap();
    memtable.set(to_key("a"), to_bytes("world")).unwrap();

    let mut sstable = SSTable::new(memtable.lock()).unwrap();

    println!("iterating from 'e'");
    i = 0;
    for (key, value) in sstable.scan(Some(to_key("e"))).unwrap() {
        println!("key: {:?}\t value: {:?}", key, value);
        if i > 15 {
            println!("oops!");
            break;
        }
        i += 1;
    }

    println!("getting: {:?}", to_key("c"));
    let mut res = sstable.get(to_key("c"));
    println!("res: {:?}", res);

    println!("getting: {:?}", to_key("i"));
    res = sstable.get(to_key("i"));
    println!("res: {:?}", res);

    println!("getting: {:?}", to_key("a"));
    res = sstable.get(to_key("a"));
    println!("res: {:?}", res);

    println!("getting: {:?}", to_key("aaa"));
    res = sstable.get(to_key("aaa"));
    println!("res: {:?}", res);
}
