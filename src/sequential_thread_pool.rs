use crate::ThreadPool;
use std::collections::HashMap;

pub struct SequentialThreadPool<Output> {
    threadpool: ThreadPool<(usize, Output)>,
    outputs: HashMap<usize, Output>,
    output_index: usize,
}

impl<Output> SequentialThreadPool<Output>
where
    Output: 'static + Send,
{
    pub fn new<Input: 'static + Send + Sync>(
        threads: usize,
        process_fn: impl Fn(Input) -> Output + 'static + Sync + Send,
        inputs: impl Iterator<Item = Input>,
    ) -> Self {
        let threadpool = ThreadPool::new(threads, move |(index, input)| (index, process_fn(input)), inputs.enumerate());
        Self {
            threadpool,
            outputs: HashMap::new(),
            output_index: 0,
        }
    }
}

impl<Output: 'static + Send> Iterator for SequentialThreadPool<Output> {
    type Item = Output;

    fn next(&mut self) -> Option<Output> {
        loop {
            if let Some(output) = self.outputs.remove(&self.output_index) {
                self.output_index += 1;
                return Some(output);
            }
            if let Some((index, output)) = self.threadpool.next() {
                self.outputs.insert(index, output);
            } else {
                return None;
            }
        }
    }
}
