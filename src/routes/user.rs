use crate::database::database::Database;
use crate::database::models::{self, format_user_id, DynamicQr, DynamicQrResult, User};
use crate::errors::{ApiError, ApiResponse, Response};
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

    if user.id != format_user_id(token.sub) {
        return Err(ApiError::Unauthorized);
    }

    match db.insert_user(user.into_inner()).await {
        Ok(user) => {
            Ok(json!({"status": Status::Created, "message": "User created. ", "data": user}))
        }
        Err(err) => Err(ApiError::InternalServerError(err.to_string())),
    }
}

#[post("/user/<user_id>/qrcode", format = "json", data = "<qrcode>")]
pub async fn create_qrcodes(
    token: Claims,
    db: &State<Database>,
    user_id: String,
    qrcode: Json<models::DynamicQr>,
) -> Response<Value> {
    if !token.has_permissions(&["write:dynamicqr"]) {
        return Err(ApiError::Unauthorized);
    }

    if user_id != format_user_id(token.sub) {
        return Err(ApiError::Unauthorized);
    }

    let url = db
        .insert_dynamic_url(&user_id, qrcode.into_inner())
        .await?;

    Ok(json!({"dynamic_url": url}))
}

#[get("/user/<user_id>/qrcode")]
pub async fn read_qrcodes(token: Claims, user_id: &str, db: &State<Database>) -> Response<Json<ApiResponse>> {
    if !token.has_permissions(&["read:dynamicqr"]) {
        return Err(ApiError::Unauthorized);
    }

    if user_id != format_user_id(token.sub) {
        return Err(ApiError::Unauthorized);
    }

    let urls = db
        .list_user_urls(&user_id)
        .await?;

    Ok(Json(ApiResponse {
        status: Status::Ok.code,
        message: "Dynamic URLs".to_string(),
        data: json!(urls),
    }))
}

#[put("/user/<user_id>/qrcode/<qrcode_id>", format = "json", data = "<qrcode>")]
pub async fn update_qrcodes(
    token: Claims,
    db: &State<Database>,
    user_id: &str,
    qrcode_id: &str,
    qrcode: Json<models::DynamicQr>,
) -> Response<Json<ApiResponse>> {
    if !token.has_permissions(&["write:dynamicqr"]) {
        return Err(ApiError::Unauthorized);
    }

    if user_id != format_user_id(token.sub) {
        return Err(ApiError::Unauthorized);
    }

    let url = db
        .update_dynamic_url(&qrcode_id, &qrcode.target_url)
        .await?;

    Ok(Json(ApiResponse {
        status: Status::Ok.code,
        message: "Dynamic URL updated".to_string(),
        data: json!(url),
    }))
}

#[delete("/user/<user_id>/qrcode/<qrcode_id>")]
pub async fn delete_qrcodes(
    token: Claims,
    db: &State<Database>,
    user_id: &str,
    qrcode_id: &str,
) -> Response<Json<ApiResponse>> {
    if !token.has_permissions(&["delete:dynamicqr"]) {
        return Err(ApiError::Unauthorized);
    }

    if user_id != format_user_id(token.sub) {
        return Err(ApiError::Unauthorized);
    }

    let url = db.delete_dynamic_url(&qrcode_id).await?;

    Ok(Json(ApiResponse {
        status: Status::Ok.code,
        message: "Dynamic URL deleted".to_string(),
        data: json!(url),
    }))
}
