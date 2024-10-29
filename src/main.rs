mod encoding;
mod guard;
mod matrix;
mod output;
mod tests;
mod utils;

use guard::{Claims};
use rocket::{get, http::Status, routes};
use rocket_cors::{AllowedOrigins, CorsOptions};
use shuttle_runtime::SecretStore;
use utils::Environments;
use rocket::serde::{Serialize, json::Json};

#[derive(Serialize)]
struct MyResponse {
    message: String,
    data: Option<String>, // Optional field for additional data
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

#[get("/")]
fn index() -> &'static str {
    "Running..."
}

#[get("/test_auth")]
fn test_auth(token: Claims) -> Result<(Status, Json<MyResponse>), (Status, Json<ErrorResponse>)> {
    let required_perms = vec!["read:all".to_string(), "write:all".to_string()];
    let perms = token.permissions.join(" ");

    // Check if all required permissions are present
    if required_perms.iter().all(|perm| token.permissions.contains(perm)) {
        let response = MyResponse {
            message: "Authorization successful".to_string(),
            data: Some(perms), // Include user permissions
        };
        Ok((Status::Ok, Json(response))) // Return JSON response on success
    } else {
        let error_response = ErrorResponse {
            error: "Unauthorized access".to_string(),
        };
        Err((Status::Unauthorized, Json(error_response))) // Return error response
    }
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
