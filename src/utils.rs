
use base64::alphabet::URL_SAFE;
use jsonwebtoken::{decode_header, Algorithm, DecodingKey, TokenData, Validation, decode};
use rocket::http::Status;
use rocket::request::{Outcome, FromRequest};
use serde::{Deserialize};
use reqwest;
use base64::{engine::general_purpose, Engine};
use std::fmt::{self, format};

// Todo: Refactor all this shit.

#[derive(Debug, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,

}

pub struct Auth(pub Claims);


#[derive(Debug, Deserialize)]
enum AuthErrors {
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

#[derive(Debug)]
enum CustomError {
    JwkNotFound,
    ReqwestError(reqwest::Error),
    MissingKid,
    DecodeError(String),
    InvalidJwk(String),
}

impl From<reqwest::Error> for CustomError {
    fn from(err: reqwest::Error) -> Self {
        CustomError::ReqwestError(err)
    }
}

fn pad_base64_url(encoded: &str) -> String {
    let mut padded = encoded.to_string();
    let pad_len = (4 - (padded.len() % 4)) % 4; // Calculate the necessary padding
    padded.push_str(&"=".repeat(pad_len)); // Add the appropriate number of padding characters
    padded
}


fn cleanse_jwk(jwk: &Jwk) -> Result<(Vec<u8>, Vec<u8>), CustomError> {
    if jwk.kty != "RSA" {
        return Err(CustomError::InvalidJwk(format!("Unsupported key type: {}", jwk.kty)));
    }

    let n_padded = pad_base64_url(&jwk.n);
    let e_padded = pad_base64_url(&jwk.e);
    
    // Decode the base64 URL encoded n and e values
    let n_bytes = general_purpose::URL_SAFE
        .decode(&n_padded)
        .map_err(|a| CustomError::InvalidJwk(format!("{}", a).to_string()))?;
    
    let e_bytes = general_purpose::URL_SAFE
        .decode(&e_padded)
        .map_err(|a| CustomError::InvalidJwk(format!("{}", a).to_string()))?;
    
    Ok((n_bytes, e_bytes))
}

async fn fetch_jwk(kid: &str) -> Result<Jwk, CustomError> {
    let jwks_url = "http://dev-211n5irc8p5i7cf6.au.auth0.com/.well-known/jwks.json";
    let jwks: Jwks = reqwest::get(jwks_url).await.map_err(CustomError::from)?.json().await.map_err(CustomError::from)?;

    match jwks.keys.iter().find(|jwk| jwk.kid == kid) {
        Some(jwk) => Ok(jwk.clone()), // Return a clone of the found JWK
        None => Err(CustomError::JwkNotFound),
    }
}

async fn decode_jwt(token: &str) -> Result<Claims, CustomError> {
    let header = decode_header(token).expect("no good");

    let kid = header.kid.ok_or("Missing Kid").expect("double bad");

    let jwk = fetch_jwk(&kid).await.unwrap();

    let (n_bytes, e_bytes) = cleanse_jwk(&jwk)?;

    let decoding_key = DecodingKey::from_rsa_raw_components(&n_bytes, &e_bytes);

    let mut validation = Validation::new(Algorithm::RS256);
    validation.set_audience(&["https://backend-test.com"]);

    let decoded: TokenData<Claims> = decode(token, &decoding_key, &validation)
        .map_err(|err| format!("Failed to decode token: {:?}", err)).expect("skibidi");

    Ok(decoded.claims)
}


    #[rocket::async_trait]
impl <'r> FromRequest<'r> for Auth {
    type Error = ();

    async fn from_request(request: &'r rocket::Request<'_>) ->  Outcome<Self, Self::Error>{
        let token = request.headers().get_one("Authorization");

        if let Some(bearer_token) = token {
            let token_str = bearer_token.trim_start_matches("Bearer ").trim();

            match decode_jwt(token_str).await  {
                Ok(claims) => Outcome::Success(Auth(claims)),
                Err(_) => Outcome::Error((Status::Unauthorized, ())),
            }
        } else {
            Outcome::Error((Status::Unauthorized, ()))
        }

      
    }
}

pub async fn test_decode(token: &str) {
    decode_jwt(token).await.expect("adsad");
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