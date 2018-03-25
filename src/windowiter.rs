use std::collections::VecDeque;

pub struct DynamicWindowIterator<V, Iter: Iterator<Item = V>> {
    iter: Iter,
    window: VecDeque<V>,
    window_offset: usize,
}

impl<V, Iter> DynamicWindowIterator<V, Iter>
where
    Iter: Iterator<Item = V>,
{
    pub fn from_iter(iter: Iter) -> DynamicWindowIterator<V, Iter> {
        DynamicWindowIterator::<V, Iter> {
            iter,
            window: VecDeque::new(),
            window_offset: 0,
        }
    }

    pub fn read_till(&mut self, idx: usize) {
        let count = idx - (self.window_offset + self.window.len());
        for _ in 0..count {
            match self.iter.next() {
                Some(item) => self.window.push_back(item),
                None => return,
            }
        }
    }

    pub fn get(&mut self, idx: usize) -> Option<&V> {
        self.read_till(idx + 1);
        self.window.get(idx - self.window_offset)
    }

    pub fn truncate(&mut self, idx: usize) {
        // Removes all history of this DynamicWindowIterator before a given index.AsMut
        self.read_till(idx);
        let count = idx - self.window_offset;
        for _ in 0..count {
            match self.window.pop_front() {
                Some(_) => self.window_offset += 1,
                None => return,
            };
        }
    }
}
