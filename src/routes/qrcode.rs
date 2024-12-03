use crate::database::database::Database;
use crate::database::models::{self, DynamicUrl, format_user_id};
use crate::errors::{ApiError, Response};
use crate::routes::guard::Claims;

use rocket::response::Redirect;
use rocket::serde::{json::Json, json::Value};
use rocket::State;
use rocket::{get, post};
use serde_json::json;

#[get("/scan/<server_url>")]
pub async fn scan(db: &State<Database>, server_url: &str) -> Response<Redirect> {
    let url = db.lookup_dynamic_url(&server_url.to_string()).await?;

    Ok(Redirect::to(format!("https://{}", url)))
}

#[post("/create_dynamic_qrcode", format = "json", data = "<qrcode>")]
pub async fn create_dynamic_qrcode(
    token: Claims,
    db: &State<Database>,
    qrcode: Json<models::DynamicUrl>,
) -> Response<Value> {
    if !token.has_permissions(&["write:dynamicqr"]) {
        return Err(ApiError::Unauthorized);
    }

    // todo: check if target_url already exists in one of users links.

    let url = db
        .insert_dynamic_url(&token.sub, qrcode.into_inner())
        .await?;

    Ok(json!({"dynamic_url": url}))
}

#[get("/list_users_dynamic_qrcodes")]
pub async fn list_users_dynamic_qrcodes(token: Claims, db: &State<Database>) -> Response<Value> {
    if !token.has_permissions(&["read:dynamicqr"]) {
        return Err(ApiError::Unauthorized);
    }

    let urls = db.list_user_urls(format_user_id(token.sub).as_str()).await?; 

    // get users sub err: unauthed, does'nt exist (auto)
    // get qr codes related to user :err: non exist
    // format to Json response
    // return

    Ok(json!({"dynamic_urls": urls}))
}


// todo: create front-end route to fetch users created qr codes and format for easy rendering.