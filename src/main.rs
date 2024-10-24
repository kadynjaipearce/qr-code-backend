mod encoding;
mod guard;
mod matrix;
mod output;
mod tests;
mod utils;

use guard::AuthGuard;
use rocket::{get, routes};
use rocket_cors::{AllowedOrigins, CorsOptions};
use shuttle_runtime::SecretStore;
use utils::Environments;

#[get("/")]
fn index() -> &'static str {
    "Running..."
}

#[get("/test_auth")]
fn test_auth(token: AuthGuard) -> String {
    format!("hello {}", token.0.sub)
}

#[shuttle_runtime::main]
async fn main(#[shuttle_runtime::Secrets] secrets: SecretStore) -> shuttle_rocket::ShuttleRocket {

    let env = Environments::new(secrets);

    let cors = CorsOptions::default()
        .allowed_origins(AllowedOrigins::all())
        .to_cors()
        .expect("Error creating CORS middleware");

    let rocket = rocket::build()
        .mount("/", routes![index, test_auth])
        .attach(cors)
        .manage(env);
        
    Ok(rocket.into())
}
