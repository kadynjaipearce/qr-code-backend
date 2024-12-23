use crate::database::database::Database;
use crate::database::models::{
    self, format_user_id, DynamicQr, DynamicQrResult, SubscriptionTier, User,
};
use crate::errors::{ApiError, ApiResponse, Response};
use crate::routes::guard::Claims;

use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::State;
use rocket::{delete, get, post, put};
use serde_json::json;

async fn validate_and_get_subscription(
    db: &State<Database>,
    user_id: &str,
) -> Result<models::UserSubscriptionResult, ApiError> {
    let subscription = db.get_subscription(user_id).await?;

    // Check if the subscription is valid (you could check subscription status or expiration here)
    if subscription.subscription_status != "complete" {
        return Err(ApiError::Unauthorized);
    }

    Ok(subscription)
}

#[post("/user", format = "json", data = "<user>")]
pub async fn create_user(
    token: Claims,
    db: &State<Database>,
    user: Json<User>,
) -> Response<Json<ApiResponse>> {
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
        Ok(user) => Ok(Json(ApiResponse {
            status: Status::Created.code,
            message: "User created".to_string(),
            data: json!({"user": user}),
        })),
        Err(err) => Err(ApiError::InternalServerError(err.to_string())),
    }
}

#[post("/user/<user_id>/qrcode", format = "json", data = "<qrcode>")]
pub async fn create_qrcodes(
    token: Claims,
    db: &State<Database>,
    user_id: String,
    qrcode: Json<models::DynamicQr>,
) -> Response<Json<ApiResponse>> {
    /*
           Creates a dynamic URL for a user.

           Params:
               user_id: the user's Auth0 ID.
               qrcode: the dynamic URL object containing the target URL.

           Returns:
               Response<Json<ApiResponse>>: the created dynamic URL object as a json response.

    */

    if user_id != format_user_id(token.sub) {
        return Err(ApiError::Unauthorized);
    }

    // Validate the user's subscription and get the subscription details

    match validate_and_get_subscription(&db, &user_id).await {
        Ok(subscription) => {
            // Check if the usage is within allowed limits for the tier

            let tier = SubscriptionTier::from_str(&subscription.tier).ok_or_else(|| {
                ApiError::InternalServerError("Invalid subscription tier".to_string())
            })?;

            if subscription.usage >= tier.max_usage() {
                return Err(ApiError::InternalServerError(
                    "Usage limit reached".to_string(),
                ));
            }
            // Create the dynamic URL
            let created = db.insert_dynamic_url(&user_id, qrcode.into_inner()).await?;

            // Increment usage after successful creation
            db.increment_usage(&user_id).await?;

            // Return a success response
            Ok(Json(ApiResponse {
                status: Status::Created.code,
                message: "Dynamic URL created".to_string(),
                data: json!({"created": created}),
            }))
        }
        Err(error) => Err(error), // Handle errors from subscription logic
    }
}

#[get("/user/<user_id>/qrcode")]
pub async fn read_qrcodes(
    token: Claims,
    user_id: &str,
    db: &State<Database>,
) -> Response<Json<ApiResponse>> {
    /*
              Lists all dynamic URLs created by a user.

              Params:
                user_id: the user's Auth0 ID.

              Returns:
                Response<Json<ApiResponse>>: the list of dynamic URLs as a json response.
    */

    if user_id != format_user_id(token.sub) {
        return Err(ApiError::Unauthorized);
    }

    match validate_and_get_subscription(&db, &user_id).await {
        Ok(_subscription) => {
            // Create the dynamic URL
            let urls = db.list_user_urls(&user_id).await?;

            // Return a success response
            Ok(Json(ApiResponse {
                status: Status::Created.code,
                message: "Dynamic Urls".to_string(),
                data: json!({"urls": urls}),
            }))
        }
        Err(error) => Err(error), // Handle errors from subscription logic
    }
}

#[put(
    "/user/<user_id>/qrcode/<qrcode_id>",
    format = "json",
    data = "<qrcode>"
)]
pub async fn update_qrcodes(
    token: Claims,
    db: &State<Database>,
    user_id: &str,
    qrcode_id: &str,
    qrcode: Json<models::DynamicQr>,
) -> Response<Json<ApiResponse>> {
    /*
           Updates a dynamic URL for a user.

           Params:
               user_id: the user's Auth0 ID.
               qrcode_id: the dynamic URL ID.
               qrcode: the dynamic URL object containing the target URL.

           Returns:
               Response<Json<ApiResponse>>: the updated dynamic URL object as a json response.

    */

    if user_id != format_user_id(token.sub) {
        return Err(ApiError::Unauthorized);
    }

    let updated = db
        .update_dynamic_url(&qrcode_id, &qrcode.target_url)
        .await?;

    Ok(Json(ApiResponse {
        status: Status::Ok.code,
        message: "Dynamic URL updated".to_string(),
        data: json!({"updated": updated}),
    }))
}

#[delete("/user/<user_id>/qrcode/<qrcode_id>")]
pub async fn delete_qrcodes(
    token: Claims,
    db: &State<Database>,
    user_id: &str,
    qrcode_id: &str,
) -> Response<Json<ApiResponse>> {
    /*
           Deletes a dynamic URL for a user.

           Params:
               user_id: the user's Auth0 ID.
               qrcode_id: the dynamic URL ID.

           Returns:
               Response<Json<ApiResponse>>: the deleted dynamic URL object as a json response.
    */

    if user_id != format_user_id(token.sub) {
        return Err(ApiError::Unauthorized);
    }

    match validate_and_get_subscription(&db, &user_id).await {
        Ok(_subscription) => {
            // Create the dynamic URL
            let deleted = db.delete_dynamic_url(&qrcode_id).await?;

            // Return a success response
            Ok(Json(ApiResponse {
                status: Status::Created.code,
                message: "Dynamic Urls".to_string(),
                data: json!({"deleted": deleted}),
            }))
        }
        Err(error) => Err(error), // Handle errors from subscription logic
    }
}
