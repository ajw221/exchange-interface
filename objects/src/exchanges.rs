use crate::{
    responses::{
        APIResponse,
        TradingPairResponse
    },
};

use base64::{
    decode,
    encode,
};

use hmac::{
    Hmac, 
    Mac, 
    NewMac
};

use reqwest::{
    header::{
        HeaderMap,
        HeaderValue,
    },
    StatusCode,
};

use sha2::{
    Digest,
    Sha256,
    Sha512,
};

use serde::de::DeserializeOwned;

use serde_json::json;

use std::{
    collections::HashMap,
    env::var,
    time::{
        SystemTime, 
        UNIX_EPOCH,
    },
};



#[derive(Debug)]
pub struct BaseExchange {
    pub api_key: String,
    pub api_secret: String,
    pub api_key_2fa: String,
    pub api_secret_2fa: String,
    pub api_passphrase: String,
    pub base_url: String,
    pub client: reqwest::Client,
    pub api_passphrase_required: Option<bool>,
}

impl Default for BaseExchange {
    fn default() -> Self {
        let client = reqwest::Client::builder().build().unwrap();
        Self {
            api_key: "".to_string(),
            api_secret: "".to_string(),
            api_key_2fa: "".to_string(),
            api_secret_2fa: "".to_string(),
            api_passphrase: "".to_string(),
            base_url: "".to_string(),
            client: client,
            api_passphrase_required: None,
        }
    }
}

impl BaseExchange {
    pub fn api_pass_required(&mut self) -> bool {
        if self.api_passphrase_required.is_none() {
            self.set_api_passphrase_required();
        }
        match self.api_passphrase_required {
            Some(val) => val,
            None => panic!("Error setting api_passphrase requirement.") 
        }
    }
    fn set_api_passphrase_required(&mut self) {
        self.api_passphrase_required = Some(
            match var("API_PASSPHRASE_REQUIRED") {
                Ok(val) => {
                    match val.is_empty() {
                        true => false,
                        false => {
                            val == "1"
                        },
                    }
                },
                Err(_) => false
            }
        )
    }
    fn nonce() -> u128 {
        let current_time = SystemTime::now();
        let since_epoch = current_time.duration_since(UNIX_EPOCH).expect("Error creating since_epoch");
        since_epoch.as_millis()
    }

    pub async fn construct_req<T>(&mut self, href: String, method: &str, mut data: HashMap<String, String>) -> Result<T, reqwest::StatusCode> 
    where 
        T: DeserializeOwned,
    {
        let api_passphrase_required: bool = match self.api_passphrase_required {
            Some(val) => val,
            None => {
                self.set_api_passphrase_required();
                self.api_passphrase_required.unwrap()
            }
        };
        let mut req_url: String = format!("{}{}",self.base_url,href);
        let data_empty: bool = data.is_empty();
        let r = match method {
            "GET" => {
                if !data_empty {
                    req_url.push_str("?");
                    let mut params: Vec<String> = Vec::new();
                    for (key, val) in data.clone().into_iter() {
                        params.push(format!("{}={}",key,val));
                    }
                    req_url.push_str(&params.join("&"));
                }
                self.client.get(req_url)
            },
            "POST" => self.client.post(req_url),
            _ => panic!("Error invalid method"),
        };
        let resp = match href.contains("private") {
            true => {
                match api_passphrase_required {
                    true => {
                        if self.api_passphrase.is_empty()  {
                            panic!("Unable to issue request due to API_PASSPHRASE missing while being reqiured.");
                        }
                    },
                    false => {
                        if data.contains_key("otp") {
                            data.remove("otp");
                        }
                    }
                }
                let json_payload = json!(data);
                let headers = self.create_headers(href, data).await;
                match data_empty {
                    true => {
                        r.headers(headers).send().await
                    },
                    false => {
                        r.headers(headers).form(&json_payload).send().await
                    },
                }
            },
            false => {r.send().await},
        };
        match &resp {
            Ok(res) => {
                if res.status() != StatusCode::OK {
                    return Err(res.status());
                }
            },
            Err(err) => {
                if err.is_status() {
                    return Err(err.status().unwrap());
                } else {
                    return Err(StatusCode::BAD_REQUEST);
                }
            }
        }

        let r = resp.unwrap().json::<T>().await;
        match r {
            Ok(r) => Ok(r),
            Err(_) => {
                Err(StatusCode::BAD_REQUEST)
            }
        }
    }

    pub async fn create_headers(&mut self, href: String, data: HashMap<String, String>) -> HeaderMap {
        let mut headers = HeaderMap::new();
        let mut nonce = Self::nonce().to_string();

        if data.contains_key("nonce") {
            nonce = data.get("nonce").unwrap().to_string();
        }
        match data.is_empty() {
            false => {
                let api_key = match self.api_pass_required() {
                    true => &self.api_key_2fa,
                    false => &self.api_key,
                };
                let api_secret = match api_key == &self.api_key_2fa {
                    true => &self.api_secret_2fa,
                    false => &self.api_secret,
                };
                let api_key_val = HeaderValue::from_str(api_key).unwrap();
                headers.insert("API-Key", api_key_val);

                let sign_result = self.build_signature(api_secret.to_string(), href, nonce, data).await;
                match sign_result {
                    Ok(api_sign) => {
                        let api_sign_val = HeaderValue::from_str(&api_sign).unwrap();
                        headers.insert("API-Sign", api_sign_val);
                    },
                    Err(_) => {
                        panic!("Error creating signature.");
                    }
                }        
            },
            true => {}
        }
        headers
    }

    pub async fn build_signature(&self, secret: String, href: String, nonce: String, payload: HashMap<String, String>) -> Result<String, Box<dyn std::error::Error>> {
        let href = format!("/0{}",href);
        let mut encoded_payload: String = String::from("");
        let mut arguments: Vec<String> = vec![format!("nonce={}",nonce)];
        for (key, value) in payload.into_iter() {
            if &key != "nonce" {
                arguments.push(format!("{}={}",key,urlencoding::encode(&value)));
            }
        }
        encoded_payload = arguments.join("&");

        let mut hashed_payload: Vec<u8> = Vec::new();
        let mut sha_digest: Sha256 = Sha256::default();
        sha_digest.update(nonce.to_string());
        sha_digest.update(encoded_payload);
        hashed_payload = sha_digest.finalize().to_vec();
        
        let secret_bytes: Vec<u8> = decode(&secret).expect("Error decoding api_secret.");
        let mut hmac_512: Hmac<Sha512> = Hmac::<Sha512>::new_varkey(&secret_bytes).expect("Error creating Hmac<Sha512>.");
        hmac_512.update(href.as_bytes());
        hmac_512.update(&hashed_payload);
        
        Ok(encode(hmac_512.finalize().into_bytes()))
    }

    pub async fn get_server_time(&mut self) -> Result<APIResponse, reqwest::StatusCode> {
        let response: Result<APIResponse, reqwest::StatusCode> = self.construct_req("/public/Time".to_string(), "GET", HashMap::new()).await;
        response
    }

    pub async fn get_tradable_asset_pairs(&mut self, pairs: Vec<String>, info: Option<String>) -> Result<TradingPairResponse, reqwest::StatusCode> {
        let mut payload: HashMap<String, String> = HashMap::new();
        payload.insert("pair".to_string(), pairs.join(","));
        if let Some(i) = info {
            payload.insert("info".to_string(), i);
        }
        let response: Result<TradingPairResponse, reqwest::StatusCode> = self.construct_req("/public/AssetPairs".to_string(), "GET", payload).await;
        response
    }

    pub async fn get_open_orders(&mut self) -> Result<APIResponse, reqwest::StatusCode> {
        let mut payload: HashMap<String, String> = HashMap::new();
        let api_passphrase = &self.api_passphrase;
        
        payload.insert("nonce".to_string(), Self::nonce().to_string());
        if !api_passphrase.is_empty() {
            payload.insert("otp".to_string(), api_passphrase.to_string());
        }

        let response: Result<APIResponse, reqwest::StatusCode> = self.construct_req("/private/OpenOrders".to_string(), "POST", payload).await;
        response
    }
}