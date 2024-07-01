use std::iter::Peekable;

use anyhow::Result;
use bytes::Bytes;
use commons::messages::KeyType;

use self::{
    iterators::LSMTreeIterator,
    memtable::{Active, Locked, Memtable},
    sstable::SSTable,
};

const MAX_MEMTABLE_SIZE_BYTES: u64 = 200;

pub mod config;
pub mod iterators;
pub mod memtable;
pub mod sstable;

pub struct LSMTree {
    active_memtable: Memtable<Active>,
    locked_memtables: Vec<Memtable<Locked>>,
    sstables: Vec<SSTable>,
}

impl LSMTree {
    pub fn new() -> Self {
        Self {
            active_memtable: Memtable::new(),
            locked_memtables: Vec::with_capacity(4),
            sstables: vec![],
        }
    }

    pub fn get(&self, key: &KeyType) -> Option<&Bytes> {
        if let Some(value) = self.active_memtable.get(key) {
            return value.as_ref();
        }
        for memtable in self.locked_memtables.iter().rev() {
            if let Some(value) = memtable.get(key) {
                return value.as_ref();
            }
        }
        None
    }

    pub fn set(&mut self, key: KeyType, value: Option<Bytes>) -> Result<()> {
        let active_size = self.active_memtable.size_bytes()?;
        let entry_size: u64 = match &value {
            Some(v) => v.len().try_into()?,
            None => 0,
        } + bincode::serialized_size(&key)?;
        if active_size + entry_size > MAX_MEMTABLE_SIZE_BYTES {
            let curr = std::mem::take(&mut self.active_memtable);
            self.locked_memtables.push(curr.lock());
        }
        self.active_memtable.set(key, value)
    }

    pub fn scan(
        &self,
        key: Option<&KeyType>,
    ) -> Peekable<Box<dyn Iterator<Item = (&KeyType, &Option<Bytes>)> + '_>> {
        let mut v = vec![];
        v.push(self.active_memtable.scan(key));
        for table in &self.locked_memtables {
            v.push(table.scan(key));
        }

        let iter: Box<dyn Iterator<Item = _>> = Box::new(LSMTreeIterator::new(v));
        iter.peekable()
    }
}

#[cfg(test)]
mod tests {
    use bytes::Bytes;
    use commons::messages::KeyType;

    use super::LSMTree;

    #[test]
    fn scan_test() {
        let mut lsm_tree = LSMTree::new();
        let to_bytes =
            |s: &str| -> Option<Bytes> { Some(Bytes::from(bincode::serialize(s).unwrap())) };
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

        for (key, value) in lsm_tree.scan(Some(&to_key("e"))) {
            println!("key: {:?}\t value: {:?}", key, value);
        }

        assert_eq!(true, true);
    }
}
