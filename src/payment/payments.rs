use std::str::FromStr;

use crate::database::database::Database;
use crate::database::models::{format_user_id, UserSubscription};
use crate::errors::{ApiError, ApiResponse, Response};
use crate::routes::guard::Claims;
use crate::utils::Environments;

use rocket::data::{FromData, ToByteUnit};
use rocket::outcome::Outcome;
use rocket::request::FromRequest;
use rocket::serde::{json::Json, json::Value};
use rocket::State;
use rocket::{delete, get, post, put};
use serde_json::json;

use rocket::http::Status;
use stripe::generated::core::token;
use stripe::{CheckoutSession, CreateCheckoutSession, CreateCustomer, Customer, EventObject};
use stripe::{Client, Subscription, SubscriptionId};
use stripe::{EventType, Webhook};

use crate::payment::models::PaymentRequest;

#[post("/subscription/<user>/create", format = "json", data = "<payment>")]
pub async fn create_checkout_session(
    token: Claims,
    payment: Json<PaymentRequest>,
    db: &State<Database>,
    user: &str,
    stripe: &State<Client>,
    env: &State<Environments>,
) -> Response<Value> {
    /*
        Creates a new checkout session for a payment.

        Params:
            payment: payment object containing the payment details.

        Returns:
            Response<Value>: the created checkout session url in a json response.

    */

    let user = match db.select_user(&format_user_id(token.sub)).await? {
        Some(user) => user,
        None => return Err(ApiError::NotFound),
    };

    // create a customer with user info.

    let customer = Customer::create(
        &stripe,
        CreateCustomer {
            name: Some(&user.username),
            email: Some(&user.email),
            metadata: Some(std::collections::HashMap::from([(
                String::from("async-stripe"),
                String::from("true"),
            )])),

            ..Default::default()
        },
    )
    .await?;

    // create a checkout session with the customer id and payment details.

    let session = CheckoutSession::create(
        &stripe,
        CreateCheckoutSession {
            cancel_url: Some("http://localhost:4200/cancel"),
            success_url: Some("http://localhost:4200/success"),
            customer: Some(customer.id),
            client_reference_id: Some(&user.id.to_string()),
            mode: Some(stripe::CheckoutSessionMode::Subscription),
            line_items: Some(vec![stripe::CreateCheckoutSessionLineItems {
                price: match payment.tier.as_str() {
                    "pro" => Some(env.get("STRIPE_PRODUCT_PRO")),
                    "lite" => Some(env.get("STRIPE_PRODUCT_LITE")),
                    _ => return Err(ApiError::BadRequest),
                },
                quantity: Some(1),
                ..Default::default()
            }]),
            expand: &["line_items", "line_items.data.price.product"],
            ..Default::default()
        },
    )
    .await?;

    // return the checkout session url.

    Ok(json!({
        "session_url": session.url,
        "session_id": session.id,
    }))
}

#[put(
    "/subscription/<user>/update",
    format = "json",
    data = "<subscription>"
)]
pub async fn update_subscription(
    token: Claims,
    user: &str,
    subscription: Json<Subscription>,
    db: &State<Database>,
    stripe: &State<Client>,
) -> Response<Value> {
    /*
        Updates a subscription for a user.

        Params:
            subscription: subscription object containing the subscription details.

        Returns:
            Response<Value>: the updated subscription object in a json response.

    */

    if !token.has_permissions(&["write:subscription"]) {
        return Err(ApiError::Unauthorized);
    }

    if user != format_user_id(token.sub) {
        return Err(ApiError::Unauthorized);
    }

    Subscription::update(
        &stripe,
        &SubscriptionId::from_str(user).unwrap(),
        stripe::UpdateSubscription {
            items: Some(vec![
                stripe::UpdateSubscriptionItems {
                    id: Some("".to_string()),
                    deleted: Some(true),
                    ..Default::default()
                },
                stripe::UpdateSubscriptionItems {
                    price: Some("".to_string()),
                    quantity: Some(1),
                    ..Default::default()
                },
            ]),
            ..Default::default()
        },
    )
    .await?;

    Ok(json!({"message": "Subscription updated."}))
}


#[delete("/subscription/<user_id>/cancel", format = "json")]
pub async fn cancel_subscription(
    token: Claims,
    user_id: &str,
    db: &State<Database>,
    stripe: &State<Client>,
) -> Response<Json<ApiResponse>> {
    /*
        Cancels a subscription for a user.

        Params:
            subscription: subscription object containing the subscription details.

        Returns:
            Response<Value>: the cancelled subscription object in a json response.

    */

    if user_id != format_user_id(token.sub) {
        return Err(ApiError::Unauthorized);
    }

    let subscription_id = match db.lookup_subscription_id(&user_id).await? {
        Some(id) => id,
        None => return Err(ApiError::NotFound),
    };

    let result = Subscription::cancel(
        &stripe,
        &SubscriptionId::from_str(&subscription_id).unwrap(),
        stripe::CancelSubscription {
            prorate: Some(true),
            invoice_now: Some(true),
            ..Default::default()
        },
    )
    .await;

    match result {
        Ok(data) => Ok(Json(ApiResponse {
            status: 200,
            message: "Subscription cancelled. ".to_string(),
            data: json!(data),
        })),
        Err(error) => {
            eprintln!("Error cancelling subscription: {:?}", error);
            return Err(ApiError::InternalServerError(error.to_string()));
        }
    }
}


#[post("/stripe/webhook", format = "json", data = "<payload>")]
pub async fn stripe_webhook(
    stripe_signature: StripeSignature<'_>,
    db: &State<Database>,
    payload: Payload,
    secrets: &State<crate::utils::Environments>,
) -> Response<Json<ApiResponse>> {
    /*
        Stripe webhook to catch new subscription events.

        Params:
            stripe_signature: stripe signature object containing the stripe signature.
            payload: payload object containing the payload details.

        Returns:
            Response<Value>: the event object in a json response.

    */

    // verify the stripe signature.

    if let Ok(event) = Webhook::construct_event(
        &payload.contents,
        stripe_signature.signature,
        &secrets.get("STRIPE_WEBHOOK_SECRET"),
    ) {
        match event.type_ {
            EventType::CheckoutSessionCompleted => {
                if let EventObject::CheckoutSession(session) = event.data.object {
                    let user = session.client_reference_id.expect("Failed to get session id. ");

                    let subscription = session.subscription.unwrap().clone();
                    let subscription_obj = subscription.as_object().unwrap();

                    let pro= &secrets.get("STRIPE_PRODUCT_PRO");
                    let lite= &secrets.get("STRIPE_PRODUCT_LITE");

                    let new_subscription = UserSubscription{
                        id: subscription_obj.id.to_string(),
                        tier: match subscription_obj.items.data[0].price.as_ref().unwrap().id.as_str() {
                            id if id == *pro => "pro".to_string(),
                            id if id == *lite => "lite".to_string(),
                            _ => return Err(ApiError::BadRequest),
                        },
                    };

                    match db.insert_subscription(&user, new_subscription).await {
                        Ok(_) => {
                           
                           unimplemented!()
                        }
                        Err(err) => {
                            eprintln!("Error inserting subscription: {:?}", err);
                            return Err(ApiError::InternalServerError(err.to_string()));
                        }
                    }


                }

                

                unimplemented!()
                
            }

            EventType::CustomerSubscriptionCreated => {
                println!("Customer subscription created: {:?}", event);

                unimplemented!()
            }

            EventType::CustomerSubscriptionPaused => {
                println!("Customer subscription paused: {:?}", event);

                unimplemented!()
            }

            EventType::CustomerSubscriptionResumed => {
                println!("Customer subscription resumed: {:?}", event);

                unimplemented!()
            }

            EventType::CustomerSubscriptionDeleted => {
                println!("Customer subscription deleted: {:?}", event);

                unimplemented!()
            }
            _ => return Ok(Json(ApiResponse {
                status: Status::PartialContent.code,
                message: "Event received. ".to_string(),
                data: json!(event),
            })),
        }
    } else {
        panic!("Error verifying stripe signature. ");
    }
}

pub struct Payload {
    pub contents: String,
}

#[rocket::async_trait]
impl<'r> FromData<'r> for Payload {
    type Error = ApiError;

    async fn from_data(
        req: &'r rocket::Request<'_>,
        data: rocket::Data<'r>,
    ) -> rocket::data::Outcome<'r, Self> {
        use rocket::outcome::Outcome;

        let limit = req
            .limits()
            .get("form")
            .unwrap_or_else(|| 1_000_000.bytes());

        let contents = match data.open(limit).into_string().await {
            Ok(string) if string.is_complete() => string.into_inner(),
            Ok(_) => {
                return Outcome::Error((
                    Status::PayloadTooLarge,
                    ApiError::InternalServerError("Payload too large. ".to_string()),
                ))
            }
            Err(error) => {
                return Outcome::Error((
                    Status::BadRequest,
                    ApiError::InternalServerError(error.to_string()),
                ))
            }
        };

        Outcome::Success(Payload { contents })
    }
}

pub struct StripeSignature<'a> {
    pub signature: &'a str,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for StripeSignature<'r> {
    type Error = &'r str;

    async fn from_request(
        req: &'r rocket::Request<'_>,
    ) -> rocket::request::Outcome<Self, Self::Error> {
        match req.headers().get_one("Stripe-Signature") {
            Some(signature) => Outcome::Success(StripeSignature { signature }),
            None => Outcome::Error((Status::InternalServerError, "No signature provided. ")),
        }
    }
}
