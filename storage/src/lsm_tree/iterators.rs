use commons::spec::{Entry, EntryIterator};
use std::iter::Peekable;

pub struct LSMTreeIterator<'a> {
    iterators: Vec<Peekable<EntryIterator<'a>>>,
}

impl<'a> LSMTreeIterator<'a> {
    pub fn new(iterators: Vec<EntryIterator<'a>>) -> Self {
        Self {
            iterators: iterators.into_iter().map(|iter| iter.peekable()).collect(),
        }
    }
}

impl<'a> Iterator for LSMTreeIterator<'a> {
    type Item = Entry;

    fn next(&mut self) -> Option<Self::Item> {
        let best = self
            .iterators
            .iter_mut()
            .enumerate()
            .filter_map(|(idx, x)| match x.peek() {
                None => None,
                Some(v) => Some((idx, v)),
            })
            .min_by_key(|(_, (k, _))| k);

        match best {
            None => None,
            Some((index, _)) => self.iterators.iter_mut().nth(index)?.next(),
        }
    }
}
