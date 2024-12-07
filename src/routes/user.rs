use crate::database::database::Database;
use crate::database::models::{format_user_id, User};
use crate::errors::{ApiError, Response};
use crate::routes::guard::Claims;

use rocket::serde::{json::Json, json::Value};
use rocket::State;
use rocket::{get, post};
use serde_json::json;

#[post("/create_user", format = "json", data = "<user>")]
pub async fn create_user(token: Claims, db: &State<Database>, user: Json<User>) -> Response<Value> {
    if token.sub != user.id {
        return Err(ApiError::Unauthorized);
    }

    match db.insert_user(user.into_inner()).await {
        Ok(user) => Ok(json!({"data": user})),
        Err(err) => Err(ApiError::InternalServerError(err.to_string())),
    }
}

#[get("/list_users_dynamic_qrcodes")]
pub async fn list_users_dynamic_qrcodes(token: Claims, db: &State<Database>) -> Response<Value> {
    if !token.has_permissions(&["read:dynamicqr"]) {
        return Err(ApiError::Unauthorized);
    }

    let urls = db
        .list_user_urls(format_user_id(token.sub).as_str())
        .await?;

    // get users sub err: unauthed, does'nt exist (auto)
    // get qr codes related to user :err: non exist
    // format to Json response
    // return

    Ok(json!({"dynamic_urls": urls}))
}

#[post("/cancel_subscription", format = "json", data = "<sub_id>")]
pub async fn cancel_subscription(
    token: Claims,
    db: &State<Database>,
    sub_id: Json<String>,
) -> Response<Value> {
    Ok(json!({"status": "success"}))
}

// test if authentication is working.
#[get("/test_auth")]
pub fn test_auth(token: Claims) -> Response<Value> {
    if !token.has_permissions(&["read:all", "write:all"]) {
        return Err(ApiError::Unauthorized);
    }

    Ok(json!({"status": "success"}))
}
