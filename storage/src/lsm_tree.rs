use std::{collections::VecDeque, sync::Arc};

use anyhow::Result;
use commons::spec::{EntryIterator, KeyType, ReadStore, ValueType, WriteStore};

use self::{
    config::Config,
    iterators::LSMTreeIterator,
    memtable::{Active, Locked, Memtable},
    sstable::SSTable,
};

pub mod config;
pub mod iterators;
pub mod memtable;
pub mod sstable;

pub struct LSMTree {
    config: Arc<Config>,
    active_memtable: Memtable<Active>,
    locked_memtables: VecDeque<Memtable<Locked>>,
    sstables: Vec<SSTable>,
}

impl LSMTree {
    pub fn new(config: Config) -> Self {
        let mut tree = Self {
            config: Arc::new(config),
            active_memtable: Memtable::new(),
            locked_memtables: VecDeque::with_capacity(0),
            sstables: vec![],
        };
        tree.locked_memtables =
            VecDeque::with_capacity(tree.config.lsm_tree_config.max_number_of_memtables);
        tree
    }

    pub fn new_default() -> Self {
        Self::new(Config::new_default())
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
        if active_size + entry_size > self.config.lsm_tree_config.max_memtable_size_bytes {
            let curr = std::mem::take(&mut self.active_memtable);
            self.locked_memtables.push_back(curr.lock());
        }
        if self.locked_memtables.len() > self.config.lsm_tree_config.max_number_of_memtables {
            let mut v = vec![];
            for table in self.locked_memtables.iter_mut() {
                v.push(table.scan(None)?);
            }

            self.sstables.push(SSTable::new(
                Arc::new(self.config.sstable_config.clone()),
                Box::new(LSMTreeIterator::new(v)),
            )?);
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
        let mut lsm_tree = LSMTree::new_default();
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
