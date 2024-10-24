use crate::utils::decode_jwt;
use rocket::http::Status;
use rocket::request::{FromRequest, Outcome};
use serde::Deserialize;
use shuttle_runtime::{SecretStore, Secrets};
use rocket::State;
use crate::utils::Environments;

#[derive(Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
}

pub struct AuthGuard(pub Claims);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AuthGuard {
    type Error = ();

    async fn from_request(request: &'r rocket::Request<'_>) -> Outcome<Self, Self::Error> {
        let token = request.headers().get_one("Authorization");

        let secrets = request.guard::<&State<Environments>>().await.unwrap();

        if let Some(bearer_token) = token {
            let token_str = bearer_token.trim_start_matches("Bearer ").trim();

            match decode_jwt(token_str, secrets.inner()).await {
                Ok(claims) => Outcome::Success(AuthGuard(claims)),
                Err(_) => Outcome::Error((Status::Unauthorized, ())),
            }
        } else {
            Outcome::Error((Status::Unauthorized, ()))
        }
    }
}
