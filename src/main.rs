mod encoding;
mod matrix;
mod output;
mod tests;

use rocket::{get, routes};
use bitvec::prelude::*;

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[shuttle_runtime::main]
async fn main() -> shuttle_rocket::ShuttleRocket {
    let mut bit_vector = BitVec::new();
    encoding::encode_to_bitvector("Hello", &mut bit_vector);
    println!("{:?}", bit_vector);
    let rocket = rocket::build().mount("/", routes![index]);

    Ok(rocket.into())

}

