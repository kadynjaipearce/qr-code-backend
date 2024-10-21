mod encoding;
mod matrix;
mod output;
mod tests;
mod utils;

use rocket::{get, post, routes, options};
use rocket_cors::{AllowedOrigins, CorsOptions};
use shuttle_runtime::SecretStore;
use utils::Auth;
use utils::test_decode;

#[get("/")]
fn index() -> &'static str {
    "Running..."
}

#[get("/test_auth")]
fn test_auth(token: Auth) -> String {
    format!("hello {}", token.0.sub)
}


#[shuttle_runtime::main]
async fn main(#[shuttle_runtime::Secrets] secrets: SecretStore) -> shuttle_rocket::ShuttleRocket {
    // Set up CORS
    println!("HERE");
    test_decode("eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCIsImtpZCI6Ik9uRjE1WS1LeGRaZzk1S2VLU2hlLSJ9.eyJpc3MiOiJodHRwczovL2Rldi0yMTFuNWlyYzhwNWk3Y2Y2LmF1LmF1dGgwLmNvbS8iLCJzdWIiOiJnb29nbGUtb2F1dGgyfDEwMzM2NTE0ODc1MzQ4MTM0MDIyOSIsImF1ZCI6WyJodHRwczovL2JhY2tlbmQtdGVzdC5jb20iLCJodHRwczovL2Rldi0yMTFuNWlyYzhwNWk3Y2Y2LmF1LmF1dGgwLmNvbS91c2VyaW5mbyJdLCJpYXQiOjE3Mjk0NDc4MTIsImV4cCI6MTcyOTUzNDIxMiwic2NvcGUiOiJvcGVuaWQgcHJvZmlsZSBlbWFpbCIsImF6cCI6IkNjTEkwS0FvZE1mNktoeTV2Q0RTWDl2TTllaFhDVk5sIn0.BgdrrxfhBtugCde7b6ULBUMjjICh069yYlAkTA3uzh_l-SUMRkI2QRgzj3IxpFJbo2F5IhrJwcqLtTYUeJTbKfN4D9rGsx6wdU_5UjudZW0uegvc9AZAX280t60kbW9_iv8RXF9GilS9rlHiYDydnAJn4HnzaBn14VxR9p_aa3DaY_q1R8681_adaACM1wtJAADXOHR85yYJtdr9S7qSY2pbaP1nRY3E9flwpt3G5-IgdXRms8MzC5ptbiVRCasyaF-Y-RPFBqQgs_LmI6GNp7jHkl1NbV8LznmKvmbSznVRbcw0OPIJy5fulSbgDl3uUZ7gSD2iHMADTR9AVhG7_Q").await;
    let cors = CorsOptions::default()
        .allowed_origins(AllowedOrigins::all())
        .to_cors()
        .expect("Error creating CORS middleware");

    let rocket = rocket::build()
        .mount("/", routes![index, test_auth])
        .attach(cors);

    Ok(rocket.into())
}


