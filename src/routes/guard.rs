use crate::utils::decode_jwt;
use crate::utils::Environments;

use rocket::http::Status;
use rocket::request::{FromRequest, Outcome};
use rocket::State;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
    pub permissions: Vec<String>,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Claims {
    type Error = ();

    async fn from_request(request: &'r rocket::Request<'_>) -> Outcome<Self, Self::Error> {
        let token = request.headers().get_one("Authorization");

        let secrets = request.guard::<&State<Environments>>().await.unwrap();

        if let Some(bearer_token) = token {
            let token_str = bearer_token.trim_start_matches("Bearer ").trim();

            match decode_jwt(token_str, secrets.inner()).await {
                Ok(claims) => Outcome::Success(claims),
                Err(_) => Outcome::Error((Status::Unauthorized, ())),
            }
        } else {
            Outcome::Error((Status::Unauthorized, ()))
        }
    }
}
