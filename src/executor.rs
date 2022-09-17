use std::{
    future::Future,
    sync::mpsc::{channel, Receiver, Sender},
    thread::{spawn, Result},
};
use tokio::runtime::Runtime;

#[derive(Debug)]
pub struct Executor {
    sender: Sender<Result<()>>,
    receiver: Receiver<Result<()>>,
}

impl Executor {
    pub fn new() -> Self {
        let (sender, receiver) = channel();
        Self { sender, receiver }
    }

    pub fn spawn_runtime<F, State>(&mut self, state: State, f: impl FnOnce(State) -> F + 'static + Send)
    where
        F: Future<Output = ()>,
        State: 'static + Send + Sync,
    {
        let sender = self.sender.clone();
        spawn(move || {
            let runtime = Runtime::new().unwrap();
            let result = spawn(move || runtime.block_on(f(state))).join();
            #[allow(unused_must_use)]
            {
                sender.send(result);
            };
        });
    }

    pub fn join(self) {
        drop(self.sender);
        while let Ok(result) = self.receiver.recv() {
            result.unwrap();
        }
    }
}
