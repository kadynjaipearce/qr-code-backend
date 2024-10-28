use crate::guard::Claims;
use base64::{engine::general_purpose, Engine};
use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, TokenData, Validation};
use reqwest;
use rocket::http::Status;
use rocket::request::{FromRequest, Outcome};
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
        self.env.get(key).unwrap()
    }
}

#[derive(Debug, Deserialize)]
pub enum AuthErrors {
    Missing,
    Invalid,
    Unauthorized,
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

pub fn cleanse_jwk(jwk: &Jwk) -> Result<(Vec<u8>, Vec<u8>), AuthErrors> {
    if jwk.kty != "RSA" {
        return Err(AuthErrors::Invalid);
    }

    let n_padded = pad_base64_url(&jwk.n);
    let e_padded = pad_base64_url(&jwk.e);

    // Decode the base64 URL encoded n and e values
    let n_bytes = general_purpose::URL_SAFE.decode(&n_padded);

    let e_bytes = general_purpose::URL_SAFE.decode(&e_padded);

    Ok((n_bytes.unwrap(), e_bytes.unwrap()))
}

async fn fetch_jwk(kid: &str) -> Result<Jwk, AuthErrors> {
    let jwks_url = "http://dev-211n5irc8p5i7cf6.au.auth0.com/.well-known/jwks.json"; // dev api to be removed
    let jwks: Jwks = reqwest::get(jwks_url).await.unwrap().json().await.unwrap();

    match jwks.keys.iter().find(|jwk| jwk.kid == kid) {
        Some(jwk) => Ok(jwk.clone()), // Return a clone of the found JWK
        None => Err(AuthErrors::Missing),
    }
}

pub async fn decode_jwt(token: &str, secrets: &Environments) -> Result<Claims, AuthErrors> {
    let header = decode_header(token).unwrap();

    let kid = header.kid.ok_or("Missing Kid").expect("double bad");

    let jwk = fetch_jwk(&kid).await.unwrap();

    let (n_bytes, e_bytes) = cleanse_jwk(&jwk)?;

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

pub fn alphanumeric_digit(character: u8) -> u16 {
    match character {
        b'0'..=b'9' => u16::from(character - b'0'),
        b'A'..=b'Z' => u16::from(character - b'A') + 10,
        b' ' => 36,
        b'$' => 37,
        b'%' => 38,
        b'*' => 39,
        b'+' => 40,
        b'-' => 41,
        b'.' => 42,
        b'/' => 43,
        b':' => 44,
        _ => 0,
    }
}
