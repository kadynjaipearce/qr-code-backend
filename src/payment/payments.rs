use std::str::FromStr;

use crate::database::database::Database;
use crate::database::models::{format_user_id, PaymentSession, UserSubscription};
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
use stripe::{
    CheckoutSession, CreateCheckoutSession, CreateCustomer, Customer, EventObject, EventType,
    Webhook,
};
use stripe::{Client, Subscription, SubscriptionId};

use crate::payment::models::PaymentRequest;

#[post("/subscription/<user_id>", format = "json", data = "<payment>")]
pub async fn create_checkout_session(
    token: Claims,
    payment: Json<PaymentRequest>,
    db: &State<Database>,
    user_id: &str,
    stripe: &State<Client>,
    secrets: &State<Environments>,
) -> Response<Json<ApiResponse>> {
    /*
        Creates a new checkout session for a payment.

        Params:
            payment: payment object containing the payment details.

        Returns:
            Response<Value>: the created checkout session url in a json response.

    */

    if user_id != format_user_id(token.sub) {
        return Err(ApiError::Unauthorized);
    }

    let user = match db.select_user(&user_id).await? {
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
                    "Pro" => Some(secrets.get("STRIPE_PRODUCT_PRO")),
                    "Lite" => Some(secrets.get("STRIPE_PRODUCT_LITE")),
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

    db.insert_session(
        user_id,
        PaymentSession {
            session_id: session.id.to_string(),
            tier: payment.tier.clone(),
        },
    )
    .await?;

    Ok(Json(ApiResponse {
        status: Status::Created.code,
        message: "Checkout session created. ".to_string(),
        data: json!(session.url),
    }))
}

#[put("/subscription/<user>", format = "json", data = "<subscription>")]
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

#[delete("/subscription/<user_id>", format = "json")]
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
            status: Status::Ok.code,
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
                    let user = db.lookup_user_from_session(&session.id).await?;

                    let subscription = match &session.subscription {
                        Some(sub) => {
                            /*
                                UserSubscription {
                                sub_id: &sub.id().to_string(),
                                tier: sub.as_object().unwrap().items.data.first().unwrap().price.clone().unwrap().nickname.unwrap().to_string(),
                                status: session.status.unwrap().to_string(),
                            }

                                 */

                            dbg!("{:?}", json!(session));
                            unimplemented!(
                                "sub: {:?}, tier: {:?}, status: {:?}",
                                &sub.id().to_string(),
                                &sub.as_object()
                                    .expect("No object")
                                    .items
                                    .data
                                    .first()
                                    .expect("No Items")
                                    .price
                                    .clone()
                                    .expect("No price")
                                    .nickname
                                    .expect("No nickname")
                                    .to_string(),
                                session.status.unwrap().to_string()
                            );
                        }
                        None => {
                            return Err(ApiError::BadRequest);
                        }
                    };

                    db.insert_subscription(&user.id.key().to_string(), subscription)
                        .await?;

                    Ok(Json(ApiResponse {
                        status: Status::Ok.code,
                        message: "Subscription inserted. ".to_string(),
                        data: json!(user),
                    }))
                } else {
                    Err(ApiError::BadRequest)
                }
            }

            EventType::CustomerSubscriptionCreated => {
                if let EventObject::Subscription(subscription) = event.data.object {
                    dbg!("{:?}", json!(subscription));
                    unimplemented!("CUSTOMER SUBSCRIPTION CREATED");
                } else {
                    return Err(ApiError::BadRequest);
                }

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
                if let EventObject::Subscription(subscription) = event.data.object {
                    let user = db.lookup_user_from_subscription(&subscription.id).await?;

                    unimplemented!("Customer subscription deleted. {:?}", user.id);
                } else {
                    return Err(ApiError::BadRequest);
                }
            }
            _ => {
                return Ok(Json(ApiResponse {
                    status: Status::PartialContent.code,
                    message: "Event received. ".to_string(),
                    data: json!(event),
                }))
            }
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
