use crate::database::database::Database;
use crate::database::models::{self, format_user_id, DynamicQr, DynamicQrResult, User};
use crate::errors::{ApiError, Response};
use crate::routes::guard::Claims;

use rocket::http::Status;
use rocket::serde::{json::Json, json::Value};
use rocket::State;
use rocket::{delete, get, post, put};
use serde_json::json;

#[post("/user", format = "json", data = "<user>")]
pub async fn create_user(token: Claims, db: &State<Database>, user: Json<User>) -> Response<Value> {
    /*
           Lists all dynamic URLs created by a user.

           only to be called by auth0 action after Auth0 post-registration.

           Params:
               user: user object containing the user's Auth0 ID and email.

           Returns:
               Response<Value>: the created user object as a json response.

    */

    let user_token = format_user_id(token.sub.clone());

    if user_token != user.id {
        return Err(ApiError::Unauthorized);
    }

    match db.insert_user(user.into_inner()).await {
        Ok(user) => {
            Ok(json!({"status": Status::Created, "message": "User created. ", "data": user}))
        }
        Err(err) => Err(ApiError::InternalServerError(err.to_string())),
    }
}

#[post("/user/<user_id>/qr_codes", format = "json", data = "<qrcode>")]
pub async fn create_qrcodes(
    token: Claims,
    db: &State<Database>,
    user_id: String,
    qrcode: Json<models::DynamicQr>,
) -> Response<Value> {
    if !token.has_permissions(&["write:dynamicqr"]) {
        return Err(ApiError::Unauthorized);
    }

    let url = db
        .insert_dynamic_url(&token.sub, qrcode.into_inner())
        .await?;

    Ok(json!({"dynamic_url": url}))
}

#[get("/user/<user_id>/qr_codes")]
pub async fn read_qrcodes(token: Claims, user_id: String, db: &State<Database>) -> Response<Value> {
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

#[put("/user/<user_id>/<qr_id>", format = "json", data = "<qrcode>")]
pub async fn update_qrcodes(
    token: Claims,
    db: &State<Database>,
    user_id: String,
    qr_id: String,
    qrcode: Json<models::DynamicQr>,
) -> Response<Value> {
    if !token.has_permissions(&["write:dynamicqr"]) {
        return Err(ApiError::Unauthorized);
    }

    let url = db
        .update_dynamic_url(&qrcode.server_url, &qrcode.target_url)
        .await?;

    Ok(json!({"updated": url}))
}

#[delete("/user/<user_id>/<qr_id>")]
pub async fn delete_qrcodes(
    token: Claims,
    db: &State<Database>,
    user_id: String,
    qr_id: String,
) -> Response<Value> {
    if !token.has_permissions(&["delete:dynamicqr"]) {
        return Err(ApiError::Unauthorized);
    }

    let url = db.delete_dynamic_url(&qr_id).await?;

    Ok(json!({"deleted": url}))
}
