
use async_trait::async_trait;

use base64::{
    decode,
    encode,
};

use cucumber::{
    given, 
    then,
    when,
    World, 
    WorldInit,
    writer::Json,
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

use serde_json::{
    Error as jsonError,
    from_str,
    to_string_pretty,
    Value,
};

use std::{
    convert::Infallible,
    env::var,
    fs::{
        File,
        read_to_string,
        self,
    },
    path::Path,
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
    exchange: BaseExchange,
    server_time_response: Option<ServerTimeResponse>,
    trading_pair: Option<TradingPairResponse>,
    open_orders: Option<OpenOrders>,
    validate_2fa: Option<Validate2FA>,
}

impl ExchangeWorld {
    async fn populate_base_keys(&mut self) {
        let exchange = &mut self.exchange;
        exchange.api_key = match var("API_KEY") {
            Ok(val) => val,
            Err(_) => "".to_string(),
        };
        exchange.api_secret = match var("API_SECRET") {
            Ok(val) => val,
            Err(_) => "".to_string(),
        };
        exchange.base_url = match var("BASE_URL") {
            Ok(val) => val,
            Err(_) => "".to_string(),
        };
        exchange.api_passphrase = match var("API_PASSPHRASE") {
            Ok(val) => {
                exchange.api_key_2fa = match var("API_KEY_2FA") {
                    Ok(val) => val,
                    Err(_) => "".to_string(),
                };
                exchange.api_secret_2fa = match var("API_SECRET_2FA") {
                    Ok(val) => val,
                    Err(_) => "".to_string(),
                };
                val
            },
            Err(_) => {
                exchange.api_key_2fa = "".to_string();
                exchange.api_secret_2fa = "".to_string();
                "".to_string()
            }
        };
    }

    async fn populate_passphrase(&mut self) {
        let exchange = &mut self.exchange;
        exchange.api_passphrase = match var("API_PASSPHRASE") {
            Ok(val) => val,
            Err(_) => "".to_string(),
        };
    }
}

#[async_trait(?Send)]
impl World for ExchangeWorld {
    type Error = Infallible;

    async fn new() -> Result<Self, Infallible> {
        Ok(Self {
            exchange: BaseExchange::default(),
            server_time_response: None,
            trading_pair: None,
            open_orders: None,
            validate_2fa: None,
        })
    }
}

#[given("an exchange instance")]
async fn create_valid_exchange(w: &mut ExchangeWorld) {
    w.exchange = BaseExchange::default();
}

#[when(regex = r"(API_KEY|API_KEY_2FA), (API_SECRET|API_SECRET_2FA), and (BASE_URL) exist")]
async fn check_env_vars_exist(w: &mut ExchangeWorld, api_key: String, api_secret: String, base_url: String) {
    match var(&api_key) {
        Ok(_) => {
            match var(&api_secret) {
                Ok(_) => {
                    match var(&base_url) {
                        Ok(_) => {
                            if &api_key == "API_KEY_2FA" {
                                w.exchange.api_passphrase_required = Some(true);
                                match var("API_PASSPHRASE") {
                                    Ok(_) => assert!(true),
                                    _ => panic!("Error retrieving API_PASSPHRASE value. API_PASSPHRASE is missing and required for this exchange instance")
                                }
                            }
                        },
                        _ => panic!("Error retrieving {} value. {} is missing and a required field for this exchange instance.",&base_url,&base_url)
                    }
                },
                _ => panic!("Error retrieving {} value. {} is missing and a required field for this exchange instance.",&api_secret,&api_secret)
            }
        },
        _ => panic!("Error retrieving {} value. {} is missing and a required field for this exchange instance.",&api_key, &api_key)
    }
}

#[then("the exchange instance keys are populated")]
async fn populate_exchange_instance(w: &mut ExchangeWorld) {
    w.populate_base_keys().await;
}

#[given("a server time request is sent")]
async fn request_server_time(w: &mut ExchangeWorld) {
    let exchange = &mut w.exchange;
    let response = exchange.get_server_time().await;
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
    let exchange = &mut w.exchange;
    let response = exchange.get_tradable_asset_pairs(vec![format!("{}{}",base,quote)], None).await;
    match response {
        Ok(res) => {
            w.trading_pair = Some(res);
        },
        Err(e) => {
            panic!("{}",e);
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

#[given(regex = "a populated exchange instance (requires|not requiring) API_PASSPHRASE")]
async fn verify_existing_exchange(w: &mut ExchangeWorld, context: String) {
    let ctx_required = context == String::from("required");
    let api_pass_required = w.exchange.api_pass_required();
    if ctx_required && api_pass_required && w.exchange.api_passphrase.is_empty() {
        panic!("Error API_PASSPHRASE environment variable missing.")
    }

    let exchange = &mut w.exchange;
    let api_passphrase = &exchange.api_passphrase;
    
    let mut API_KEY_VAR = "API_KEY".to_string();
    let mut API_SECRET_VAR = "API_SECRET".to_string();
    
    if ctx_required {
        API_KEY_VAR.push_str("_2FA");
        API_SECRET_VAR.push_str("_2FA");
    }

    let BASE_URL = match var("BASE_URL") {
        Ok(val) => val,
        Err(_) => "".to_string(),
    };
    let API_KEY = match var(API_KEY_VAR) {
        Ok(val) => val,
        Err(_) => "".to_string(),
    };
    let API_SECRET = match var(API_SECRET_VAR) {
        Ok(val) => val,
        Err(_) => "".to_string(),
    };
    if exchange.base_url != BASE_URL {
        panic!("Invalid BASE_URL value on exchange.");
    } else if exchange.base_url.is_empty() {
        panic!("Error BASE_URL is a required for an exchange instance.")
    }
    if ctx_required {
        let API_PASSPHRASE = match var("API_PASSPHRASE") {
            Ok(val) => val,
            Err(_) => panic!("API_PASSPHRASE is required for this exchange instance.")
        };
        if exchange.api_key_2fa != API_KEY {
            panic!("Invalid API_KEY value on exchange.");
        } else if exchange.api_secret_2fa != API_SECRET {
            panic!("Invalid API_SECRET value on exchange.");
        } else {
            match api_passphrase.is_empty() {
                true => {
                    exchange.api_passphrase = API_PASSPHRASE;
                },
                false => {
                    if api_passphrase != &API_PASSPHRASE {
                        panic!("Error API_PASSPHRASE mismatch.");
                    }
                }
            }
        }
    } else {
        if exchange.api_key != API_KEY {
            panic!("Invalid API_KEY value on exchange.");
        } else if exchange.api_secret != API_SECRET {
            panic!("Invalid API_SECRET value on exchange.");
        }
    }
}

#[when(expr = "an open orders request is sent and a response is received with {int} errors")]
async fn open_orders_request_sent_response_received(w: &mut ExchangeWorld, errors: usize) {
    let exchange = &mut w.exchange;
    let response = exchange.get_open_orders().await;
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

#[then(expr = "the response should contain the OpenOrders result")]
async fn validate_open_orders_response(w: &mut ExchangeWorld) {
    let open_orders = &w.open_orders;
    if let None = open_orders {
        panic!("Invalid Open Orders object received.")
    }
}

#[tokio::main]
async fn main() {
    let mut public_features_path = String::from("/public_features");
    let mut private_features_path = String::from("/private_features");
    if !Path::new(&public_features_path).exists() {
        public_features_path = format!("src{}",&public_features_path.to_string());
        private_features_path = format!("src{}",&private_features_path.to_string());
    }
    ExchangeWorld::cucumber().with_writer(Json::new(File::create("public_features_report.json").unwrap())).run(&public_features_path).await;
    ExchangeWorld::cucumber().with_writer(Json::new(File::create("private_features_report.json").unwrap())).run(&private_features_path).await;

    format_json_file("public_features_report.json");
    format_json_file("private_features_report.json");
}

fn format_json_file(filename: &str) {
    let file_string = read_to_string(filename);
    match file_string {
        Ok(file_str) => {
            let result: Result<Value, jsonError> = from_str(&file_str);
            match result {
                Ok(value) => {
                    let beautified = to_string_pretty(&value);
                    match beautified {
                        Ok(b_string) => {
                            fs::write(filename, b_string).expect("Error writing to file.");
                        },
                        Err(_) => panic!("Error retrieving pretty string result.")
                    }
                },
                Err(_) => panic!("Error retrieving Value result.")
            }
        },
        Err(_) => panic!("Error reading file to string.")
    }
}