use crate::database::database::Database;
use crate::database::models::User;
use crate::errors::{ApiError, Response};
use crate::routes::guard::Claims;
use rocket::serde::{json::Json, json::Value, Deserialize, Serialize};
use rocket::State;
use rocket::{get, post};
use serde_json::json;

#[post("/create_user", format = "json", data = "<user>")]
pub async fn create_user(db: &State<Database>, user: Json<User>) -> Response<Value> {
    let auth0_id = user.id.clone();

    match db.select_user(auth0_id).await? {
        Some(existing_user) => {
            return Err(ApiError::InternalServerError(format!(
                "User already exists: {}",
                existing_user.email
            )));
        }
        None => match db.insert_user(user.into_inner()).await {
            Ok(user) => Ok(json!({"data": user})),
            Err(err) => {
                return Err(ApiError::InternalServerError(format!(
                    "Database error during user creation: {}",
                    err
                )));
            }
        },
    }
}

#[get("/validate")]
pub async fn validate_user(db: &State<Database>) -> Response<Value> {
    let users = db.validate_user("enxrm@gmail.com".to_string()).await?;

    Ok(json!({"user": users}))
}

#[post("/find_user", format = "json", data = "<id>")]
pub async fn find_user(db: &State<Database>, id: Json<String>) -> Response<Value> {
    let result = db.select_user(id.to_string()).await?;

    Ok(json!({"user": result}))
}

#[get("/test_auth")]
pub fn test_auth(token: Claims) -> Response<Value> {
    if !token.has_permissions(&["read:all", "write:all"]) {
        return Err(ApiError::Unauthorized);
    }

    Ok(json!({"status": "success"}))
}
