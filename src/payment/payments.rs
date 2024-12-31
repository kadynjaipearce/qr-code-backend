use crate::database::database::Database;
use crate::database::models::{
    format_user_id, PaymentSession, SubscriptionAction, UpdateRequest, UserSubscription,
};
use crate::errors::{ApiError, ApiResponse, Response};
use crate::payment::models::PaymentRequest;
use crate::routes::guard::Claims;
use crate::utils::Environments;

use rocket::data::{FromData, ToByteUnit};
use rocket::http::Status;
use rocket::outcome::Outcome;
use rocket::request::FromRequest;
use rocket::serde::json::Json;
use rocket::State;
use rocket::{delete, post, put};
use serde_json::json;
use std::str::FromStr;
use stripe::{
    CheckoutSession, CreateCheckoutSession, CreateCustomer, Customer, EventObject, EventType,
    Object, Webhook,
};
use stripe::{Client, Subscription, SubscriptionId};

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
            client_reference_id: Some(&payment.tier.to_string()),
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

#[put("/subscription/<user_id>", format = "json", data = "<update_request>")]
pub async fn update_subscription(
    token: Claims,
    update_request: Json<UpdateRequest>,
    db: &State<Database>,
    user_id: &str,
    stripe: &State<Client>,
) -> Response<Json<ApiResponse>> {
    /*
        Updates a subscription for a user.

        Params:
            subscription: subscription object containing the subscription details.

        Returns:
            Response<Value>: the updated subscription object in a json response.

    */

    if user_id != format_user_id(token.sub) {
        return Err(ApiError::Unauthorized);
    }

    let subscription_id = match db.get_subscription_id(&user_id).await? {
        Some(id) => id,
        None => return Err(ApiError::NotFound),
    };

    match update_request.action {
        SubscriptionAction::Cancel => {
            let cancelled = Subscription::update(
                &stripe,
                &SubscriptionId::from_str(&subscription_id).unwrap(),
                stripe::UpdateSubscription {
                    cancel_at_period_end: Some(true),
                    ..Default::default()
                },
            )
            .await?;

            return Ok(Json(ApiResponse {
                status: Status::Ok.code,
                message: "Subscription cancelled. ".to_string(),
                data: json!({"cancelled": cancelled}),
            }));
        }

        SubscriptionAction::Upgrade => {
            unimplemented!()
        }

        SubscriptionAction::Downgrade => {
            unimplemented!()
        }

        SubscriptionAction::Resume => {
            let resumed = Subscription::update(
                &stripe,
                &SubscriptionId::from_str(&subscription_id).unwrap(),
                stripe::UpdateSubscription {
                    cancel_at_period_end: Some(false),
                    ..Default::default()
                },
            )
            .await?;

            return Ok(Json(ApiResponse {
                status: Status::Ok.code,
                message: "Subscription cancelled. ".to_string(),
                data: json!({"resumed": resumed}),
            }));
        }
    }
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

    let subscription_id = match db.get_subscription_id(&user_id).await? {
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
    .await?;

    Ok(Json(ApiResponse {
        status: Status::Ok.code,
        message: "Subscription cancelled. ".to_string(),
        data: json!({"cancelled": result}),
    }))
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

    if let Ok(event) = Webhook::construct_event(
        &payload.contents,
        stripe_signature.signature,
        &secrets.get("STRIPE_WEBHOOK_SECRET"),
    ) {
        match event.type_ {
            EventType::CheckoutSessionCompleted => {
                if let EventObject::CheckoutSession(session) = event.data.object {
                    let user = db.get_user_from_session(&session.id).await?;

                    dbg!(&user);

                    let _subscription = match &session.subscription {
                        Some(sub) => {
                            let subscription = db
                                .insert_subscription(
                                    &user.id.key().to_string(),
                                    UserSubscription {
                                        sub_id: sub.id().to_string(),
                                        tier: session.client_reference_id.unwrap().to_string(),
                                        status: session.status.unwrap().to_string(),
                                    },
                                )
                                .await?;

                            dbg!(&user.id.key().to_string());

                            return Ok(Json(ApiResponse {
                                status: Status::Ok.code,
                                message: "Subscription inserted. ".to_string(),
                                data: json!({"subscribed": subscription}),
                            }));
                        }
                        None => {
                            return Err(ApiError::BadRequest);
                        }
                    };
                } else {
                    Err(ApiError::BadRequest)
                }
            }

            EventType::CustomerSubscriptionDeleted => {
                if let EventObject::Subscription(subscription) = event.data.object {
                    let user = db.get_user_from_subscription(&subscription.id).await?;

                    let _ = db.delete_user_data(&user.id.key().to_string()).await?;

                    return Ok(Json(ApiResponse {
                        status: Status::Ok.code,
                        message: "Subscription deleted. ".to_string(),
                        data: json!({"deleted": subscription.id().to_string()}),
                    }));
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
        return Err(ApiError::InternalServerError(
            "Stripe signature invalid. ".to_string(),
        ));
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
