pub struct Batcher<T: Clone> {
    offset: usize,
    size: usize,
    items: Vec<T>,
}

impl<T: Clone> Batcher<T> {
    pub fn new(items: Vec<T>, size: usize) -> Self {
        Self { items, size, offset: 0 }
    }
}

impl<T: Clone> Iterator for Batcher<T> {
    type Item = Vec<T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.offset < self.items.len() {
            let end_offset = std::cmp::min(self.offset + self.size, self.items.len());
            let items = self.items[self.offset..end_offset].to_vec();
            self.offset = end_offset;
            Some(items)
        } else {
            None
        }
    }
}
