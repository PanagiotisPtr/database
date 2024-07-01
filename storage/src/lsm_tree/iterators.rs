use std::iter::Peekable;

use bytes::Bytes;
use commons::messages::KeyType;

pub struct LSMTreeIterator<'a, I>
where
    I: Iterator<Item = (&'a KeyType, &'a Option<Bytes>)>,
{
    iterators: Vec<Peekable<I>>,
}

impl<'a, I> LSMTreeIterator<'a, I>
where
    I: Iterator<Item = (&'a KeyType, &'a Option<Bytes>)>,
{
    pub fn new(iterators: Vec<Peekable<I>>) -> Self {
        Self { iterators }
    }
}

impl<'a, I> Iterator for LSMTreeIterator<'a, I>
where
    I: Iterator<Item = (&'a KeyType, &'a Option<Bytes>)>,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        if self.iterators.len() == 0 {
            return None;
        }
        let mut iters = self.iterators.iter_mut();
        let mut best: &mut Peekable<I> = iters.next().unwrap();
        for iter in iters {
            if let Some(curr) = iter.peek() {
                if let Some(prev) = best.peek() {
                    if curr < prev {
                        best = iter;
                    }
                } else {
                    best = iter;
                }
            }
        }
        best.next()
    }
}
