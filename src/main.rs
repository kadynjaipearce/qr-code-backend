mod database;
mod errors;
mod payment;
mod routes;
mod tests;
mod utils;

use rocket::{get, routes};
use rocket_cors::{AllowedOrigins, CorsOptions};
use shuttle_runtime::SecretStore;
use utils::Environments;

#[get("/")]
fn index() -> &'static str {
    r#"
    Welcome to the API!

    How to Use:
    This API is intended to be accessed only from the frontend.

    1. Make sure to make requests from the frontend (browser, client-side, etc.).
    2. The backend only accepts requests from the frontend, any direct requests from tools like Postman or Curl will be rejected.
    
    CORS (Cross-Origin Resource Sharing) must be properly configured to allow these requests.

    Stay tuned for more API documentation and features in the future!

    Thank you for using our API!
    "#
}

#[shuttle_runtime::main]
async fn main(#[shuttle_runtime::Secrets] secrets: SecretStore) -> shuttle_rocket::ShuttleRocket {
    let env = Environments::new(secrets);
    let db = database::database::Database::new(&env).await.unwrap();
    let stripe = stripe::Client::new(env.get("STRIPE_SECRET"));

    let cors = CorsOptions::default()
        .allowed_origins(AllowedOrigins::all())
        .to_cors()
        .expect("Error creating CORS middleware");

    let rocket = rocket::build()
        .mount(
            "/api",
            routes![
                index,
                routes::qrcode::scan,
                routes::user::create_user,
                routes::user::create_dynamic_qrcode,
                routes::user::read_dynamic_qrcode,
                routes::user::update_dynamic_qrcode,
                payment::payments::create_checkout_session,
            ],
        )
        .attach(cors)
        .manage(env)
        .manage(db)
        .manage(stripe);

    Ok(rocket.into())
}
