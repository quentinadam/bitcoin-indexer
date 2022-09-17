use std::{env, error, net::IpAddr};

#[derive(Debug, Clone)]
pub struct Configuration {
    host: IpAddr,
    port: u16,
    threads: usize,
    batch_size: usize,
    block_files_path: String,
    store_file_path: String,
    confirmations: usize,
    update_interval: u64,
    rpc_server_host: String,
    rpc_server_port: u16,
    rpc_server_user: String,
    rpc_server_password: String,
}

fn var(key: &str, default: Option<String>) -> Result<String, String> {
    match env::var(key) {
        Ok(value) => Ok(value),
        Err(_) => default.ok_or(format!("Missing {}", key)),
    }
}

fn var_map<T, E: error::Error>(key: &str, mut f: impl FnMut(&str) -> Result<T, E>, default: Option<T>) -> Result<T, String> {
    match var(key, None) {
        Ok(value) => f(&value).map_err(|_| format!("Invalid {} {}", key, value)),
        Err(err) => default.ok_or(err),
    }
}

impl Configuration {
    pub fn new() -> Self {
        let host = var_map("HOST", |host| host.parse(), Some("127.0.0.1".parse().unwrap())).unwrap();
        let port = var_map("PORT", |port| port.parse(), Some(8000)).unwrap();
        let threads = var_map("THREADS", |threads| threads.parse(), None).unwrap();
        let batch_size = var_map("BATCH_SIZE", |threads| threads.parse(), None).unwrap();
        let store_file_path = var("STORE_FILE_PATH", None).unwrap();
        let block_files_path = var("BLOCK_FILES_PATH", None).unwrap();
        let update_interval = var_map("UPDATE_INTERVAL", |interval| interval.parse(), Some(1000)).unwrap();
        let confirmations = var_map("CONFIRMATIONS", |confirmations| confirmations.parse(), Some(6)).unwrap();
        let rpc_server_host = var("RPC_SERVER_HOST", None).unwrap();
        let rpc_server_port = var_map("RPC_SERVER_PORT", |port| port.parse(), None).unwrap();
        let rpc_server_user = var("RPC_SERVER_USER", None).unwrap();
        let rpc_server_password = var("RPC_SERVER_PASSWORD", None).unwrap();
        Self {
            host,
            port,
            threads,
            batch_size,
            store_file_path,
            block_files_path,
            update_interval,
            confirmations,
            rpc_server_host,
            rpc_server_port,
            rpc_server_user,
            rpc_server_password,
        }
    }

    pub fn host(&self) -> IpAddr {
        self.host
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn threads(&self) -> usize {
        self.threads
    }

    pub fn batch_size(&self) -> usize {
        self.batch_size
    }

    pub fn block_files_path(&self) -> &str {
        &self.block_files_path
    }

    pub fn store_file_path(&self) -> &str {
        &self.store_file_path
    }

    pub fn confirmations(&self) -> usize {
        self.confirmations
    }

    pub fn update_interval(&self) -> u64 {
        self.update_interval
    }

    pub fn rpc_server_host(&self) -> &str {
        &self.rpc_server_host
    }

    pub fn rpc_server_port(&self) -> u16 {
        self.rpc_server_port
    }

    pub fn rpc_server_user(&self) -> &str {
        &self.rpc_server_user
    }

    pub fn rpc_server_password(&self) -> &str {
        &self.rpc_server_password
    }
}
