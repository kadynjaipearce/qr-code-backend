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
        Ok(user) => Ok(json!({"data": user})),
        Err(err) => Err(ApiError::InternalServerError(err.to_string())),
    }
}

#[post("/cancel_subscription", format = "json", data = "<sub_id>")]
pub async fn cancel_subscription(
    token: Claims,
    db: &State<Database>,
    sub_id: Json<String>,
) -> Response<Value> {
    /*



    */
    !unimplemented!()
}
