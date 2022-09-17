use std::{
    sync::{
        mpsc::{channel, Receiver},
        Arc, Mutex,
    },
    thread::spawn,
};

pub struct ThreadPool<Output> {
    output_receiver: Receiver<Output>,
}

impl<Output> ThreadPool<Output>
where
    Output: 'static + Send,
{
    pub fn new<Input: 'static + Send + Sync>(
        threads: usize,
        process_fn: impl Fn(Input) -> Output + 'static + Sync + Send,
        inputs: impl Iterator<Item = Input>,
    ) -> Self {
        let (input_sender, input_receiver) = channel::<Input>();
        let input_receiver = Arc::new(Mutex::new(input_receiver));
        let (output_sender, output_receiver) = channel();

        let process_fn = Arc::new(process_fn);
        for _ in 0..threads {
            let input_receiver = input_receiver.clone();
            let output_sender = output_sender.clone();
            let process_fn = process_fn.clone();
            spawn(move || loop {
                let input = input_receiver.lock().unwrap().recv();
                match input {
                    Ok(input) => {
                        let output = process_fn(input);
                        output_sender.send(output).unwrap();
                    }
                    Err(_) => {
                        break;
                    }
                }
            });
        }
        for input in inputs {
            input_sender.send(input).unwrap()
        }
        Self { output_receiver }
    }
}

impl<Output> Iterator for ThreadPool<Output> {
    type Item = Output;

    fn next(&mut self) -> Option<Self::Item> {
        self.output_receiver.recv().ok()
    }
}
