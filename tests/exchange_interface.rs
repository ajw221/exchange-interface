
use async_trait::async_trait;

use base64::{
    decode,
    encode,
};

use core::env;

use cucumber::{
    given, 
    then,
    when,
    World, 
    WorldInit
};

use hmac::{
    Hmac,
    Mac,
    NewMac,
};

use objects::{
    exchanges::{
        BaseExchange
    },
    orders::OpenOrders,
    responses::{
        ResponseResult,
        TradingPairResponse,
    },
    system_server::ServerTime,
};

use sha2::{
    Digest,
    Sha256,
    Sha512,
};

use std::{
    convert::Infallible,
    env::var,
    time::{
        SystemTime,
        UNIX_EPOCH,
    },
};



#[derive(Debug)]
pub struct ServerTimeResponse {
    server_time: ServerTime,
    current_ts: i64,
}

#[derive(Debug)]
pub struct Validate2FA {
    private_key: String,
    nonce: String,
    endpoint: String,
}

impl Validate2FA {
    fn sign(&self, sign_to_match: String) -> bool {
        let nonce = &self.nonce;
        let payload = format!("{}nonce={}",nonce,nonce);

        let mut hashed_payload: Vec<u8> = Vec::new();
        let mut sha_digest: Sha256 = Sha256::default();
        sha_digest.update(nonce.to_string());
        sha_digest.update(payload.to_string());
        hashed_payload = sha_digest.finalize().to_vec();

        let pk_bytes: Vec<u8> = decode(&self.private_key).expect("Error decoding private_key.");
        let mut hmac_512: Hmac<Sha512> = Hmac::<Sha512>::new_varkey(&pk_bytes).expect("Error creating Hmac<Sha512>.");
        hmac_512.update(self.endpoint.as_bytes());
        hmac_512.update(&hashed_payload);
        
        let signed = encode(hmac_512.finalize().into_bytes());

        signed == sign_to_match
    }
}

#[derive(Debug, WorldInit)]
pub struct ExchangeWorld {
    exchange: Option<BaseExchange>,
    server_time_response: Option<ServerTimeResponse>,
    trading_pair: Option<TradingPairResponse>,
    open_orders: Option<OpenOrders>,
    validate_2fa: Option<Validate2FA>,
}

impl ExchangeWorld {
    async fn populate_passphrase(&mut self) {
        let exchange = &mut self.exchange;
        if let Some(ex_obj) = exchange {
            ex_obj.api_passphrase = Ok(env!("API_PASSPHRASE").to_string());
        }
    }
}

#[async_trait(?Send)]
impl World for ExchangeWorld {
    type Error = Infallible;

    async fn new() -> Result<Self, Infallible> {
        Ok(Self {
            exchange: None,
            server_time_response: None,
            trading_pair: None,
            open_orders: None,
            validate_2fa: None,
        })
    }
}

#[given("an exchange instance")]
async fn create_valid_exchange(w: &mut ExchangeWorld) {
    w.exchange = Some(BaseExchange::default());

    #[when(regex = r"(API_KEY), (API_SECRET), and (BASE_URL) exist")]
    async fn check_env_vars_exist(w: &mut ExchangeWorld, api_key: String, api_secret: String, base_url: String) {
        match var(&api_key) {
            Ok(_) => {
                match var(&api_secret) {
                    Ok(_) => {
                        match var(&base_url) {
                            Ok(_) => {

                                #[then("the exchange instance keys are populated")]
                                async fn create_exchange_instance(w: &mut ExchangeWorld) {
                                    let exchange = &mut w.exchange;
                                    match exchange {
                                        Some(ex) => {
                                            ex.api_key = env!("API_KEY").to_string();
                                            ex.api_secret = env!("API_SECRET").to_string();
                                            ex.base_url = env!("BASE_URL").to_string();
                                        },
                                        _ => panic!("Error retrieving Exchange instance.")
                                    }
                                }
                            },
                            _ => panic!("Error retrieving BASE_URL value.")
                        }
                    },
                    _ => panic!("Error retrieving API_SECRET value.")
                }
            },
            _ => panic!("Error retrieving API_KEY value.")
        }
    }
}

#[given("a server time request is sent")]
async fn request_server_time(w: &mut ExchangeWorld) {
    let exchange = w.exchange.as_ref();
    if let Some(ex_obj) = exchange {
        let response = ex_obj.get_server_time().await;
        let current_ts = SystemTime::now().duration_since(UNIX_EPOCH).ok().unwrap().as_secs() as i64;
        match response {
            Ok(res) => {
                let result = res.result;
                match result {
                    ResponseResult::ServerTimeResponse(r) => {
                        let server_time_response = ServerTimeResponse {
                            server_time: r,
                            current_ts: current_ts,
                        };
                        w.server_time_response = Some(server_time_response);
                    },
                    _ => panic!("Invalid ResponseResult received.")
                }
            },
            Err(e) => {
                panic!("{}",e);
            }
        }
    }
}

#[when("a server time response is received")]
async fn server_time_response_received(w: &mut ExchangeWorld) {
    let server_time_response = &w.server_time_response;
    match server_time_response {
        Some(_) => assert!(true),
        None => panic!("Server Time response not received.")
    }
}

#[then("the response time in minutes should equal the server time in minutes")]
async fn current_ts_matches_server_time(w: &mut ExchangeWorld) {
    let server_time_response = &w.server_time_response;
    match server_time_response {
        Some(st_resp) => {
            assert_eq!(st_resp.server_time.unixtime / 60, st_resp.current_ts / 60);
        },
        None => panic!("Error retrieving server time response.")
    }
}

#[given(regex = r"a (?P<base>[A-Z]{3})/(?P<quote>[A-Z]{3}) trading pair request is sent")]
async fn request_trading_pair(w: &mut ExchangeWorld, base: String, quote: String) {
    let exchange = w.exchange.as_ref();
    if let Some(ex_obj) = exchange {
        let response = ex_obj.get_tradable_asset_pairs(vec![format!("{}{}",base,quote)], None).await;
        match response {
            Ok(res) => {
                w.trading_pair = Some(res);
            },
            Err(e) => {
                panic!("{}",e);
            }
        }
    }
}

#[when(regex = r"a (?P<base>[A-Z]{3})/(?P<quote>[A-Z]{3}) trading pair response is received")]
async fn trading_pair_response_received(w: &mut ExchangeWorld, base: String, quote: String) {
    if w.trading_pair.is_none() {
        panic!("Error retrieving {}/{} trading pair.",base,quote);
    }
}

#[then(regex = "the response should contain (?P<base>[A-Z]{3})/(?P<quote>[A-Z]{3}) asset pair information")]
async fn trading_pairs_validate(w: &mut ExchangeWorld, base: String, quote: String) {
    let trading_pair_results = &mut w.trading_pair.as_ref().unwrap().result.values();

    let trading_pair = match trading_pair_results.len() {
        1 => {
            trading_pair_results.nth(0).unwrap()
        },
        _ => panic!("Error retrieving first trading pair result.")
    };

    if trading_pair.altname != format!("{}{}",base,quote) {
        panic!("Invalid trading pair.")
    }
}

#[given("API_PASSPHRASE exists")]
async fn check_api_passphrase(w: &mut ExchangeWorld) {
    match var("API_PASSPHRASE") {
        Ok(_) => {
            w.populate_passphrase().await;
        },
        Err(e) => {
            panic!("{}",e);
        }
    }
}

#[when(expr = "using {word}, {word}, and {word} for sign testing")]
async fn gathering_signing_values(w: &mut ExchangeWorld, private_key: String, nonce: String, endpoint: String) {
    let validate_2fa = Validate2FA {
        private_key: private_key,
        nonce: nonce,
        endpoint: endpoint,
    };
    w.validate_2fa = Some(validate_2fa);
}

#[then(expr = "the resulting value should be equal to {word}")]
async fn validating_signed_value(w: &mut ExchangeWorld, signed: String) {
    let validate_2fa = &mut w.validate_2fa.as_ref().unwrap();
    if !validate_2fa.sign(signed) {
        panic!("Error validating 2FA sign.")
    }
}

#[given("a populated exchange instance contains API_PASSPHRASE")]
async fn verify_existing_exchange(w: &mut ExchangeWorld) {
    let exchange = &mut w.exchange.as_ref().unwrap();
    let api_passphrase = &exchange.api_passphrase;

    if exchange.api_key != env!("API_KEY") {
        panic!("Invalid API_KEY value on exchange.");
    } else if exchange.api_secret != env!("API_SECRET") {
        panic!("Invalid API_SECRET value on exchange.");
    } else if exchange.base_url != env!("BASE_URL") {
        panic!("Invalid BASE_URL value on exchange.");
    } else if let Ok(api_passphrase) = api_passphrase {
        match api_passphrase.is_empty() {
            true => {
                w.populate_passphrase().await;
            },
            false => {
                if api_passphrase != env!("API_PASSPHRASE") {
                    panic!("Error API_PASSPHRASE mismatch.");
                }
            }
        }
    } else {
        panic!("API_PASSWORD missing from exchange due to errors initializing instance.");
    }
}

#[when(expr = "an open orders request is sent and a response is received with {int} errors")]
async fn open_orders_request_sent_response_received(w: &mut ExchangeWorld, errors: usize) {
    let exchange = w.exchange.as_ref();
    if let Some(ex_obj) = exchange {
        let response = ex_obj.get_open_orders().await;
        match response {
            Ok(res) => {
                if errors != res.error.len() {
                    panic!("Errors received while requesting orders");
                }
                match res.result {
                    ResponseResult::OpenOrdersResponse(r) => {
                        w.open_orders = Some(r);
                    },
                    _ => panic!("Invalid ResponseResult received.")
                }
            },
            Err(e) => {
                panic!("OPEN ORDERS ERROR: {}",e);
            }
        }
    }
}

#[then(expr = "the response should contain the OpenOrders result")]
async fn validate_open_orders_response(w: &mut ExchangeWorld) {
    let open_orders = &w.open_orders;
    if let None = open_orders {
        panic!("Invalid Open Orders object received.")
    }
}

#[tokio::main]
async fn main() {
    ExchangeWorld::run("tests/public_features").await;
    ExchangeWorld::run("tests/private_features").await;
}