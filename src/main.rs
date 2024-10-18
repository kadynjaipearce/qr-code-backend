mod encoding;
mod matrix;
mod output;
mod tests;
mod utils;

use rocket::{get, post, routes};
use bitvec::prelude::*;
use shuttle_runtime::SecretStore;

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}


#[post("/test-auth")]
fn authenticate_request() {

}

#[shuttle_runtime::main]
async fn main(#[shuttle_runtime::Secrets] secrets: SecretStore) -> shuttle_rocket::ShuttleRocket {
    let mut bit_vector = BitVec::new();
    encoding::encode_to_bitvector("Hello!", &mut bit_vector);
    println!("{:?}", bit_vector);

    let rocket = rocket::build().mount("/", routes![index]);

    Ok(rocket.into())

}

