use crate::database::database::Database;
use crate::database::models::User;
use crate::errors::{ApiError, Response};
use crate::routes::guard::Claims;
use rocket::serde::{json::Json, json::Value};
use rocket::State;
use rocket::{get, post};
use serde_json::json;

#[post("/create_user", format = "json", data = "<user>")]
pub async fn create_user(token: Claims, db: &State<Database>, user: Json<User>) -> Response<Value> {
    let auth0_id = user.id.clone();
    let claim_id = token.sub;

    match db.insert_user(user.into_inner()).await {
        Ok(user) => Ok(json!({"data": user})),
        Err(err) => {
            return Err(ApiError::InternalServerError(format!(
                "Database error during user creation: {}",
                err
            )));
        }
    }
}


#[get("/test_auth")]
pub fn test_auth(token: Claims) -> Response<Value> {
    if !token.has_permissions(&["read:all", "write:all"]) {
        return Err(ApiError::Unauthorized);
    }

    Ok(json!({"status": "success"}))
}
