use crate::{base64, hex, reverse_hex, Logger, Transaction};
use hyper::body::HttpBody;
use serde_json::{json, Value};
use std::{str, error, fmt};
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct Client {
    uri: String,
    client: hyper::Client<hyper::client::HttpConnector>,
    authorization: String,
}

#[derive(Debug,Serialize, Deserialize)]
struct Error {
    code: isize,
    message: String,
}

impl error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} (code {})", self.message, self.code)
    }
}


impl Client {
    pub fn new(host: &str, port: u16, user: &str, password: &str) -> Self {
        Self {
            uri: format!("http://{}:{}", host, port),
            client: hyper::Client::new(),
            authorization: format!("Basic {}", base64::encode(format!("{}:{}", user, password))),
        }
    }

    async fn request(&self, method: &str, params: Vec<Value>, logger: &Logger) -> Result<Value, Error> {
        logger.log(format!("{} {}", method, json!(params).to_string()));
        let request = hyper::Request::builder()
            .method(hyper::Method::POST)
            .uri(&self.uri)
            .header("content-type", "application/json")
            .header("authorization", &self.authorization)
            .body(hyper::Body::from(
                json!({
                    "jsonrpc": "1.0", "id": "indexer", "method": method, "params": params
                })
                .to_string(),
            ))
            .unwrap();
        let mut response = self.client.request(request).await.unwrap();
        let mut body = Vec::new();
        while let Some(chunk) = response.body_mut().data().await {
            body.extend_from_slice(&chunk.unwrap());
        }
        match response.status() {
            hyper::StatusCode::OK => {
                let mut json: Value = serde_json::from_slice(&body).unwrap();
                Ok(json["result"].take())
            }
            hyper::StatusCode::INTERNAL_SERVER_ERROR => {
                let mut json: Value = serde_json::from_slice(&body).unwrap();
                Err(serde_json::from_value(json["error"].take()).unwrap())
            }
            _ => {
                panic!("{} {}", response.status(), str::from_utf8(&body).unwrap());
            }
        }
    }

    pub async fn getrawmempool(&self, logger: &Logger) -> Vec<[u8; 32]> {
        let result = self.request("getrawmempool", vec![json!(false)], logger).await.unwrap();
        let result: Vec<String> = serde_json::from_value(result).unwrap();
        result
            .iter()
            .map(|hash| {
                let mut result = [0u8; 32];
                reverse_hex::decode_into(hash, &mut result).unwrap();
                result
            })
            .collect()
    }

    pub async fn getrawtransaction(&self, hash: &[u8; 32], logger: &Logger) -> Option<Transaction> {
        match self.request("getrawtransaction", vec![json!(reverse_hex::encode(hash))], logger).await {
            Ok(result) => {
                let result: String = serde_json::from_value(result).unwrap();
                let buffer = hex::decode(&result).unwrap();
                Some(Transaction::from_slice(&buffer))
            },
            Err(error) => {
                if error.code == -5 {
                    None
                } else {
                    Err(error).unwrap()
                }
            }
        }
    }

    pub async fn getblockhash(&self, height: usize, logger: &Logger) -> Option<Vec<u8>> {
        match self.request("getblockhash", vec![json!(height)], logger).await {
            Ok(result) => {
                let hash: String = serde_json::from_value(result).unwrap();
                Some(reverse_hex::decode(hash).unwrap())
            }
            Err(error) => {
                if error.code == -8 {
                    None
                } else {
                    Err(error).unwrap()
                }
            }
        }
    }

    pub async fn getblock(&self, hash: impl AsRef<[u8]>, logger: &Logger) -> Option<Vec<u8>> {
        let result = self
            .request("getblock", vec![json!(reverse_hex::encode(hash)), json!(0)], logger)
            .await
            .unwrap();
        let result: String = serde_json::from_value(result).unwrap();
        Some(hex::decode(&result).unwrap())
    }
}
