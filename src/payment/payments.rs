use crate::database::database::Database;
use crate::database::models::{format_user_id, User};
use crate::errors::{ApiError, Response};
use crate::routes::guard::Claims;

use rocket::serde::{json::Json, json::Value};
use rocket::State;
use rocket::{get, post};
use serde_json::json;
use stripe::Client;
use stripe::{CheckoutSession, CreateCheckoutSession, CreateCustomer, Customer};

use crate::payment::models::PaymentRequest;

#[post("/create_checkout_session", format = "json", data = "<payment>")]
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

    // create a customer with user info.

    let customer = Customer::create(
        &stripe,
        CreateCustomer {
            name: Some(&payment.user.username),
            email: Some(&payment.user.email),
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
