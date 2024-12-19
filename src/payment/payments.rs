use std::str::FromStr;

use crate::database::database::Database;
use crate::database::models::format_user_id;
use crate::errors::{ApiError, Response};
use crate::routes::guard::Claims;

use rocket::data::{FromData, ToByteUnit};
use rocket::outcome::Outcome;
use rocket::request::FromRequest;
use rocket::serde::{json::Json, json::Value};
use rocket::State;
use rocket::{delete, post};
use serde_json::json;

use stripe::{CheckoutSession, CreateCheckoutSession, CreateCustomer, Customer};
use stripe::{Client, Subscription, SubscriptionId};
use rocket::http::Status;
use stripe::{EventType, Webhook};

use crate::payment::models::PaymentRequest;

#[post(
    "/subscription/create/<user>",
    format = "json",
    data = "<subscription>"
)]
pub async fn create_subscription(
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

    // get the user from the database.

    match user == token.sub {
        false => return Err(ApiError::Unauthorized),
        true => {
            let user = !unimplemented!(); // todo: get user subscription from the database.

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
    }
}

// test comment

#[delete("/subscription/cancel/<id>", format = "json")]
pub async fn cancel_subscription(
    token: Claims,
    id: &str,
    db: &State<Database>,
    stripe: &State<Client>,
) -> Response<Value> {
    /*
        Cancels a subscription for a user.

        Params:
            subscription: subscription object containing the subscription details.

        Returns:
            Response<Value>: the cancelled subscription object in a json response.

    */

    // get the user from the database.

    if id != format_user_id(token.sub) {
        return Err(ApiError::Unauthorized);
    }

    let user_id = db.lookup_subscription_id(&id).await?;

    let result = Subscription::cancel(
        &stripe,
        &SubscriptionId::from_str("sub_1QURyaHWw3wpkqUxbBy7nPxf").unwrap(),
        stripe::CancelSubscription {
            prorate: Some(true),
            invoice_now: Some(true),
            ..Default::default()
        },
    )
    .await;

    match result {
        Ok(data) => Ok(json!({"message": "Subscription cancelled.", "data": data})),
        Err(err) => {
            eprintln!("Error cancelling subscription: {:?}", err);
            return Err(ApiError::InternalServerError(err.to_string()));
        }
    }
}

#[post(
    "/subscription/create_checkout_session",
    format = "json",
    data = "<payment>"
)]
pub async fn create_checkout_session(
    token: Claims,
    payment: Json<PaymentRequest>,
    db: &State<Database>,
    stripe: &State<Client>,
    env: &State<crate::utils::Environments>,
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

// todo: implement webhook to update user subscription status.

/*
    impl: webhook to catch new subscription events.

    then impl: event handler for CheckoutSessionCompleted
*/

#[post("/stripe/webhook", format = "json", data = "<payload>")]
pub async fn stripe_webhook(
    stripe_signature: StripeSignature<'_>,
    payload: Payload,
    secrets: &State<crate::utils::Environments>,
) -> Response<Value> {
    /*
        Stripe webhook to catch new subscription events.

        Params:
            stripe_signature: stripe signature object containing the stripe signature.
            payload: payload object containing the payload details.

        Returns:
            Response<Value>: the event object in a json response.

    */

    // verify the stripe signature.

    if let Ok(event) = Webhook::construct_event(&payload.contents, stripe_signature.signature, &secrets.get("STRIPE_WEBHOOK_SECRET")) {
        match event.type_ {
            EventType::CheckoutSessionCompleted => {
                println!("Checkout session completed: {:?}", event);

                unimplemented!()
            },

            EventType::CustomerSubscriptionCreated => {
                println!("Customer subscription created: {:?}", event);

                unimplemented!()
            },

            EventType::CustomerSubscriptionPaused => {
                println!("Customer subscription paused: {:?}", event);

                unimplemented!()
            },

            EventType::CustomerSubscriptionResumed => {
                println!("Customer subscription resumed: {:?}", event);

                unimplemented!()
            },

            EventType::CustomerSubscriptionDeleted => {
                println!("Customer subscription deleted: {:?}", event);

                unimplemented!()
            },
            _ => return Ok(json!(event)),
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
            Ok(_) => return Outcome::Error((Status::PayloadTooLarge, ApiError::InternalServerError("Payload too large. ".to_string()))),
            Err(error) => {
                return Outcome::Error((
                    Status::ImATeapot,
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
