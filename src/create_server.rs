use crate::{json, reverse_hex, Address, Arc, HashMap, Server, State, TransactionOutput};
use hyper::{Body, Response, StatusCode};
use std::{error, fmt};

macro_rules! unwrap {
    ( $x:expr ) => {{
        match $x {
            Ok(result) => result,
            Err(error) => return respond_error(error),
        }
    }};
}

#[derive(Debug)]
pub struct Error {
    message: String,
}

impl Error {
    pub fn new(message: impl Into<String>) -> Self {
        Self { message: message.into() }
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", &self.message)
    }
}

impl error::Error for Error {}

fn parse_address(address: &str) -> Result<Address, Error> {
    Address::from_string(&address).map_err(|_| Error::new(format!(r#"Invalid addresses "{}""#, &address)))
}

fn parse_address_from_parameters(parameters: &HashMap<String, String>) -> Result<Address, Error> {
    parse_address(parameters.get("address").unwrap())
}

fn parse_addresses_from_parameters(parameters: &HashMap<String, String>) -> Result<Vec<Address>, Error> {
    let addresses = parameters
        .get("addresses")
        .ok_or_else(|| Error::new("Missing addresses parameter"))?;
    addresses.split(',').map(parse_address).collect()
}

fn parse_addresses_from_body(body: &Vec<u8>) -> Result<Vec<Address>, Error> {
    let body = std::str::from_utf8(body).map_err(|_| Error::new("Invalid utf8 body"))?;
    let json: serde_json::Value = serde_json::from_str(body).map_err(|_| Error::new("Invalid JSON body"))?;
    let array = json.as_array().ok_or_else(|| Error::new("Expecting array in JSON body"))?;
    array
        .iter()
        .map(|address| match address.as_str() {
            Some(address) => parse_address(address),
            None => Err(Error::new("Expecting array of strings in JSON body")),
        })
        .collect()
}

fn parse_confirmations(parameters: &HashMap<String, String>, max_confirmations: usize) -> Result<usize, Error> {
    match parameters.get("confirmations") {
        Some(confirmations) => match confirmations.parse::<usize>() {
            Ok(confirmations) => {
                if confirmations <= max_confirmations {
                    Ok(confirmations)
                } else {
                    Err(Error::new(format!(
                        "Expecting confirmations parameter to be less or equal to {}",
                        max_confirmations
                    )))
                }
            }
            Err(_) => Err(Error::new(format!(r#"Invalid confirmations parameter "{}""#, confirmations))),
        },
        None => Ok(0),
    }
}

fn format_value(balance: u64) -> f64 {
    (balance as f64) / 1e8
}

fn format_utxo(utxo: &TransactionOutput, value: u64) -> serde_json::Value {
    json!({
        "hash": reverse_hex::encode(utxo.hash),
        "vout": utxo.index,
        "value": format_value(value)
    })
}

async fn get_balance(state: Arc<State>, address: &Address, parameters: &HashMap<String, String>) -> Response<Body> {
    let confirmations = unwrap!(parse_confirmations(parameters, state.confirmations()));
    respond_ok(json!(format_value(state.balance(&address, confirmations).await)))
}

async fn get_balance_array(state: Arc<State>, addresses: Vec<Address>, parameters: &HashMap<String, String>) -> Response<Body> {
    let confirmations = unwrap!(parse_confirmations(parameters, state.confirmations()));
    respond_ok(json!(state
        .balance_array(addresses, confirmations)
        .await
        .iter()
        .map(|(address, balance)| json!({"address": address.to_string(), "balance": (*balance as f64)/1e8}))
        .collect::<Vec<_>>()))
}

async fn get_utxos_array(state: Arc<State>, addresses: Vec<Address>, parameters: &HashMap<String, String>) -> Response<Body> {
    let confirmations = unwrap!(parse_confirmations(parameters, state.confirmations()));
    respond_ok(json!(state
        .transaction_outputs_array(addresses, confirmations)
        .await
        .iter()
        .map(
            |(address, utxos)| json!({"address": address.to_string(), "utxos": utxos.iter().map(|(utxo, value)| format_utxo(utxo, *value)).collect::<Vec<_>>()})
        )
        .collect::<Vec<_>>()))
}

async fn get_utxos(state: Arc<State>, address: &Address, parameters: &HashMap<String, String>) -> Response<Body> {
    let confirmations = unwrap!(parse_confirmations(parameters, state.confirmations()));
    let mut utxos = Vec::new();
    state
        .iterate_transaction_outputs(&address, confirmations, |utxo, value| {
            utxos.push(format_utxo(utxo, value));
        })
        .await;
    respond_ok(json!(utxos))
}

fn respond(status: StatusCode, value: serde_json::Value) -> Response<Body> {
    Response::builder()
        .header("Content-Type", "application/json")
        .status(status)
        .body::<hyper::Body>(value.to_string().into())
        .unwrap()
}

fn respond_ok(value: serde_json::Value) -> Response<Body> {
    respond(StatusCode::OK, value)
}

fn respond_error(error: Error) -> Response<Body> {
    respond(StatusCode::BAD_REQUEST, json!({"message": error.to_string()}))
}

pub fn create_server(state: Arc<State>) -> Server<State> {
    let mut server = Server::new(state);

    server.get("/addresses/{address}/balance", |_request, parameters, _body, state| async move {
        let address = unwrap!(parse_address_from_parameters(&parameters));
        get_balance(state, &address, &parameters).await
    });

    server.get("/addresses/{address}/utxos", |_request, parameters, _body, state| async move {
        let address = unwrap!(parse_address_from_parameters(&parameters));
        get_utxos(state, &address, &parameters).await
    });

    server.get("/addresses/balance", |_request, parameters, _body, state| async move {
        let addresses = unwrap!(parse_addresses_from_parameters(&parameters));
        get_balance_array(state, addresses, &parameters).await
    });

    server.post("/addresses/balance", |_request, parameters, body, state| async move {
        let addresses = unwrap!(parse_addresses_from_body(&body));
        get_balance_array(state, addresses, &parameters).await
    });

    server.get("/addresses/utxos", |_request, parameters, _body, state| async move {
        let addresses = unwrap!(parse_addresses_from_parameters(&parameters));
        get_utxos_array(state, addresses, &parameters).await
    });

    server.post("/addresses/utxos", |_request, parameters, body, state| async move {
        let addresses = unwrap!(parse_addresses_from_body(&body));
        get_utxos_array(state, addresses, &parameters).await
    });

    server
}
