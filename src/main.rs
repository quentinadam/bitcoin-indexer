pub mod address;
pub mod base58;
pub mod base58_check;
pub mod base64;
pub mod base_binary;
pub mod base_common;
pub mod batcher;
pub mod block;
pub mod block_file_reader;
pub mod buffer_writer;
pub mod chronometer;
pub mod client;
pub mod configuration;
pub mod create_server;
pub mod error;
pub mod executor;
pub mod hashing_buffer_reader;
pub mod hashmap;
pub mod hex;
pub mod last_blocks;
pub mod logger;
pub mod reverse_hex;
pub mod sequential_thread_pool;
pub mod server;
pub mod state;
pub mod store;
pub mod thread_pool;
pub mod transaction;
pub mod transaction_output;

use self::{
    address::Address,
    batcher::Batcher,
    block::{iterate_transactions, Block, BlockHeader, BlockTrait},
    block_file_reader::BlockFileReader,
    buffer_writer::BufferWriter,
    chronometer::Chronometer,
    client::Client,
    configuration::Configuration,
    create_server::create_server,
    error::Error,
    executor::Executor,
    hashing_buffer_reader::{Hasher, HashingBufferReader},
    hashmap::{AddressHashMap, TransactionOutputHashMap},
    last_blocks::LastBlocks,
    logger::{Logger, PartialLogger},
    sequential_thread_pool::SequentialThreadPool,
    server::Server,
    state::State,
    store::{
        AugmentedStore, AugmentedTransactionStore, IndexedStore, IndexedTransactionStoreBackend, Store, TransactionStoreAugmentation,
        TransactionStoreBackendTrait,
    },
    thread_pool::ThreadPool,
    transaction::Transaction,
    transaction_output::TransactionOutput,
};
use serde_json::{self, json};
use std::{
    collections::{HashMap, HashSet, VecDeque},
    convert::TryInto,
    net::SocketAddr,
    sync::Arc,
    time::SystemTime,
};
use tokio::{
    sync::{Mutex, RwLock},
    time::{sleep, Duration},
};

fn main() {
    let configuration = Configuration::new();

    let logger = Logger::new();
    let store = match IndexedStore::from_file(configuration.store_file_path(), &logger) {
        Some(store) => store,
        None => {
            let reader = BlockFileReader::new(configuration.block_files_path());
            let mut blocks = reader.blocks(configuration.threads(), &logger);
            blocks.truncate(blocks.len() - configuration.confirmations() + 1);
            let store = IndexedStore::from_blocks(blocks, configuration.threads(), configuration.batch_size(), logger);
            store.to_file(configuration.store_file_path(), &logger);
            store
        }
    };

    let client = Client::new(
        configuration.rpc_server_host(),
        configuration.rpc_server_port(),
        configuration.rpc_server_user(),
        configuration.rpc_server_password(),
    );

    let state = Arc::new(State::new(store, client, configuration.confirmations()));

    let mut executor = Executor::new();

    executor.spawn_runtime(state.clone(), |state| async move {
        state.update().await;
    });

    executor.join();

    let mut executor = Executor::new();

    let (tx, mut rx) = tokio::sync::mpsc::channel(256);

    executor.spawn_runtime(state.clone(), {
        let configuration = configuration.clone();
        |state| async move {
            loop {
                sleep(Duration::from_millis(configuration.update_interval())).await;
                if state.update().await {
                    tx.send(()).await.unwrap();
                }
            }
        }
    });

    executor.spawn_runtime(state.clone(), {
        let configuration = configuration.clone();
        |state| async move {
            loop {
                rx.recv().await;
                state.write(configuration.store_file_path()).await;
            }
        }
    });

    executor.spawn_runtime(state.clone(), {
        let configuration = configuration.clone();
        |state| async move {
            let server = create_server(state);
            let address = SocketAddr::new(configuration.host(), configuration.port());
            server.run(address).await;
        }
    });

    executor.join();
}
