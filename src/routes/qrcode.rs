use crate::database::database::Database;
use crate::database::models::{self, format_user_id};
use crate::errors::{ApiError, Response};
use crate::routes::guard::Claims;

use rocket::response::Redirect;
use rocket::serde::{json::Json, json::Value};
use rocket::{form, put, uri, State};
use rocket::{get, post};
use serde_json::json;

#[get("/scan/<server_url>")]
pub async fn scan(server_url: &str, db: &State<Database>) -> Response<Redirect> {
    /*
       Redirects to the target URL of a dynamic QR code.

       Params:
           server_url (str): The server URL of the dynamic QR code.

       Returns:
           Response<Redirect>: Redirects to the target URL.

    */

    let url = db.lookup_dynamic_url(&server_url.to_string()).await?;

    if url.contains("Https://") || url.contains("http://") {
        return Ok(Redirect::to(url));
    }

    Ok(Redirect::to(format!("http://{}", url)))
}


