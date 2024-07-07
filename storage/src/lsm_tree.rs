use std::collections::VecDeque;

use anyhow::Result;
use commons::spec::{EntryIterator, KeyType, ReadStore, ValueType, WriteStore};

use self::{
    iterators::LSMTreeIterator,
    memtable::{Active, Locked, Memtable},
    sstable::SSTable,
};

const MAX_MEMTABLE_SIZE_BYTES: u64 = 200;
const MAX_MEMTABLES: usize = 4;

pub mod config;
pub mod iterators;
pub mod memtable;
pub mod sstable;

pub struct LSMTree {
    active_memtable: Memtable<Active>,
    locked_memtables: VecDeque<Memtable<Locked>>,
    sstables: Vec<SSTable>,
}

impl LSMTree {
    pub fn new() -> Self {
        Self {
            active_memtable: Memtable::new(),
            locked_memtables: VecDeque::with_capacity(MAX_MEMTABLES),
            sstables: vec![],
        }
    }
}

impl<'a> ReadStore<'a> for LSMTree {
    fn get(&'a mut self, key: KeyType) -> Result<ValueType> {
        if let Some(value) = self.active_memtable.get(key.clone())? {
            return Ok(Some(value));
        }
        for memtable in self.locked_memtables.iter_mut().rev() {
            if let Some(value) = memtable.get(key.clone())? {
                return Ok(Some(value));
            }
        }
        for sstable in self.sstables.iter_mut().rev() {
            if let Some(value) = sstable.get(key.clone())? {
                return Ok(Some(value));
            }
        }
        Ok(None)
    }

    fn scan(&'a mut self, key: Option<KeyType>) -> Result<EntryIterator<'a>> {
        let mut v: Vec<EntryIterator<'a>> = vec![];
        v.push(self.active_memtable.scan(key.clone())?);
        for table in self.locked_memtables.iter_mut() {
            v.push(table.scan(key.clone())?);
        }
        for table in self.sstables.iter_mut() {
            v.push(table.scan(key.clone())?);
        }

        Ok(Box::new(LSMTreeIterator::new(v)))
    }
}

impl<'a> WriteStore<'a> for LSMTree {
    fn set(&'a mut self, key: KeyType, val: ValueType) -> Result<()> {
        let active_size = self.active_memtable.size_bytes()?;
        let entry_size: u64 = match &val {
            Some(v) => v.len().try_into()?,
            None => 0,
        } + bincode::serialized_size(&key)?;
        if active_size + entry_size > MAX_MEMTABLE_SIZE_BYTES {
            let curr = std::mem::take(&mut self.active_memtable);
            self.locked_memtables.push_back(curr.lock());
        }
        if self.locked_memtables.len() > MAX_MEMTABLES {
            let mut v = vec![];
            for table in self.locked_memtables.iter_mut() {
                v.push(table.scan(None)?);
            }

            self.sstables
                .push(SSTable::new(Box::new(LSMTreeIterator::new(v)))?);
            self.locked_memtables.clear();
        }
        self.active_memtable.set(key, val)
    }
}

#[cfg(test)]
mod tests {
    use bytes::Bytes;
    use commons::spec::{KeyType, ReadStore, WriteStore};

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

        for (key, value) in lsm_tree.scan(Some(to_key("e"))).unwrap() {
            println!("key: {:?}\t value: {:?}", key, value);
        }

        assert_eq!(true, true);
    }
}
