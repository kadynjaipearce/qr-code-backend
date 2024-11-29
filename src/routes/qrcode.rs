use crate::database::database::Database;
use crate::database::models::{self, DynamicUrl};
use crate::errors::{ApiError, Response};
use crate::routes::guard::Claims;

use rocket::response::Redirect;
use rocket::serde::{json::Json, json::Value};
use rocket::State;
use rocket::{get, post};
use serde_json::json;

#[get("/scan/<server_url>")]
pub async fn scan(db: &State<Database>, server_url: String) -> Response<Redirect> {
    let url = db.lookup_dynamic_url(server_url).await?;

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
        .insert_dynamic_url(token.sub, qrcode.into_inner())
        .await?;

    Ok(json!({"dynamic_url": url}))
}
