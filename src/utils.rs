use crate::errors::{ApiError, Response};
use crate::routes::guard::Claims;

use base64::{engine::general_purpose, Engine};
use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, TokenData, Validation};
use reqwest;
use serde::Deserialize;
use shuttle_runtime::SecretStore;

#[derive(Clone)]
pub struct Environments {
    pub env: SecretStore,
}

impl Environments {
    pub fn new(secrets: SecretStore) -> Self {
        Environments { env: secrets }
    }

    pub fn get(&self, key: &str) -> String {
        match self.env.get(key) {
            Some(value) => value,
            None => panic!("Missing Key: {}", key),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
struct Jwk {
    alg: String,
    kty: String,
    kid: String,
    n: String,
    e: String,
}

#[derive(Debug, Deserialize)]
struct Jwks {
    keys: Vec<Jwk>,
}

pub fn pad_base64_url(encoded: &str) -> String {
    let mut padded = encoded.to_string();
    let pad_len = (4 - (padded.len() % 4)) % 4; // Calculate the necessary padding
    padded.push_str(&"=".repeat(pad_len)); // Add the appropriate number of padding characters
    padded
}

pub fn cleanse_jwk(jwk: &Jwk) -> Response<(Vec<u8>, Vec<u8>)> {
    if jwk.kty != "RSA" {
        return Err(ApiError::InternalServerError("Invalid Token".to_string()));
    }

    let n_padded = pad_base64_url(&jwk.n);
    let e_padded = pad_base64_url(&jwk.e);

    // Decode the base64 URL encoded n and e values
    let n_bytes = general_purpose::URL_SAFE.decode(&n_padded);

    let e_bytes = general_purpose::URL_SAFE.decode(&e_padded);

    Ok((n_bytes.unwrap(), e_bytes.unwrap()))
}

async fn fetch_jwk(kid: &str, secrets: &Environments) -> Response<Jwk> {
    let jwks_url = secrets.get("AUTH0_KNOWN_JWKS");
    let response = reqwest::get(jwks_url)
        .await
        .map_err(|_| ApiError::InternalServerError("Failed to fetch JWK".to_string()))?;

    if response.status() != reqwest::StatusCode::OK {
        eprintln!("Received non-200 response: {}", response.status());
        return Err(ApiError::InternalServerError("Failed to".to_string()));
    }

    let body = response
        .text()
        .await
        .map_err(|_| ApiError::InternalServerError("Invalid Body".to_string()))?;

    let jwks: Jwks = serde_json::from_str(&body)
        .map_err(|_| ApiError::InternalServerError("Invalid Key".to_string()))?;

    jwks.keys
        .iter()
        .find(|jwk| jwk.kid == kid)
        .cloned()
        .ok_or(ApiError::InternalServerError("Missing Key".to_string()))
}

pub async fn decode_jwt(token: &str, secrets: &Environments) -> Response<Claims> {
    let header = decode_header(token).unwrap();

    let kid = header.kid.ok_or("Missing Kid").expect("double bad");

    let jwk = fetch_jwk(&kid, &secrets).await.map_err(|err| {
        eprint!("Error Fetching: {:?}", err);
        ApiError::Unauthorized
    });

    let a = &jwk.unwrap();

    let (n_bytes, e_bytes) = cleanse_jwk(&a)?;

    let decoding_key = DecodingKey::from_rsa_raw_components(&n_bytes, &e_bytes);

    let mut validation = Validation::new(Algorithm::RS256);

    let audience = secrets.get("AUTH0_AUDIENCE");

    validation.validate_exp = true;
    validation.set_audience(&[audience]);

    let decoded: TokenData<Claims> = decode(token, &decoding_key, &validation)
        .map_err(|err| format!("Failed to decode token: {:?}", err))
        .unwrap();

    Ok(decoded.claims)
}
