mod database;
mod errors;
mod routes;
mod payment;
mod tests;
mod utils;

use rocket::{get, routes};
use rocket_cors::{AllowedOrigins, CorsOptions};
use shuttle_runtime::SecretStore;
use utils::Environments;

#[get("/")]
fn index() -> &'static str {
    "Running..."
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
            "/",
            routes![
                index,
                routes::user::create_user,
                routes::qrcode::create_dynamic_qrcode,
                routes::qrcode::scan,
                routes::qrcode::read_dynamic_qrcode,
                routes::qrcode::update_dynamic_qrcode,

            ],
        )
        .attach(cors)
        .manage(env)
        .manage(db)
        .manage(stripe);

    Ok(rocket.into())
}
