use std::collections::HashMap;

use axum::extract::{Request, State};
use axum::http::{HeaderMap, StatusCode};
use axum::middleware::Next;
use axum::response::Response;
use axum_extra::headers::{Authorization, HeaderMapExt};
use axum_extra::headers::authorization::Bearer;
use jsonwebtoken::{Algorithm, decode, decode_header, DecodingKey, Validation};
use log::debug;
use serde::{Deserialize, Serialize};

type DecodingKeys = HashMap<String, DecodingKey>;


#[derive(Debug, Serialize, Deserialize, Clone)]
struct Claims {
    upn: String,
    roles: Vec<String>,
}


pub async fn auth(
    State(decoding_keys): State<DecodingKeys>,
    headers: HeaderMap,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let claims: Option<Claims> = headers.typed_get()
        .and_then(|auth_header| extract_and_validate_token(auth_header, decoding_keys));
    return match claims {
        None => Err(StatusCode::UNAUTHORIZED),
        Some(claims) => {
            request.extensions_mut().insert(claims);
            let reps = next.run(request).await;
            Ok(reps)
        }

    };
}

fn extract_and_validate_token(auth_header: Authorization<Bearer>, decoding_keys: DecodingKeys) -> Option<Claims> {
    let token = auth_header.token();
    debug!("Token: {:?}", token);
    return validate_token(token, decoding_keys);
}

fn validate_token(token: &str, decoding_keys: DecodingKeys) -> Option<Claims> {
    let header = decode_header(token).unwrap();
    let kid = header.kid.unwrap();
    debug!("Found kid: {:?}", kid);
    match decoding_keys.get(&kid) {
        None => {
            debug!("No key found for kid: {:?}", kid);
            return None;
        }
        Some(key) => {
            debug!("Found key for kid: {:?}", kid);
            let mut validation = Validation::new(Algorithm::RS256);
            validation.set_audience(&["api://places.cluster.azure.ludimus.de"]);
            let token_data = decode::<Claims>(token, key, &validation).unwrap();
            let claims = token_data.claims;
            debug!("Claims: {:?}", claims);
            return Some(claims);
        }
    }
}


#[derive(Deserialize)]
struct Keys {
    keys: Vec<Key>,
}

#[derive(Deserialize)]
struct Key {
    kid: String,
    #[serde(rename = "use")] use_: String,
    kty: String,
    e: String,
    n: String,
}

pub async fn load_auth_public_keys() -> DecodingKeys {
    let url = "https://login.microsoftonline.com/b2748d0a-856e-4184-bda8-831f9ffa8a48/discovery/keys?appid=a4b8584b-9fbd-4bc0-bbfb-363589f1743b";
    let res = reqwest::get(url).await.unwrap();
    let keys: Keys = res.json().await.unwrap();

    let mut map = HashMap::new();

    keys.keys.iter()
        .filter(|key| key.use_ == "sig" && key.kty == "RSA")
        .for_each(|key| {
            debug!("Loaded key: {:?}", key.kid);
            map.insert(key.kid.clone(), DecodingKey::from_rsa_components(&key.n, &key.e).unwrap());
        });

    return map;
}