#[macro_use]
extern crate rocket;

use std::path::PathBuf;

use serde::{Serialize, Deserialize};
use rocket::{http::Status, request::FromRequest, request::Outcome};
use lazy_static::lazy_static;

lazy_static! {
    static ref MAP_DIR: PathBuf = {
        std::env::args().skip(1).nth(0).unwrap_or(".").into()
    };
}

fn compare_string(left: &str, right: &str, strict: bool) -> bool {
    if strict {
        left == right
    } else {
        left.contains(right)
    }
}

#[derive(Debug, Serialize)]
struct Entry {
    ip: String,
    name: String,
    clan: String,
    servername: String,
    gametype: String,
    version: i32,
    flags: i32,
    vs_bots: bool,
    timestamp: i64,
    test_db: bool,
}

struct ApiKey(String);

#[derive(Debug)]
enum ApiKeyError {
    Missing,
    Invalid,
    BadCount,
}

/// Returns true if `key` is a valid API key string.
fn is_valid(key: &str) -> bool {
    std::fs::read_to_string("./apikeys.txt")
        .unwrap_or_default()
        .lines()
        .any(|line| key == line)
}

#[rocket::async_trait]
impl<'a, 'r> FromRequest<'a, 'r> for ApiKey {
    type Error = ApiKeyError;

    async fn from_request(
        request: &'a rocket::Request<'r>,
    ) -> rocket::request::Outcome<Self, Self::Error> {
        if std::env::args().any(|x| x == "--dev") {
            return Outcome::Success(ApiKey("".to_string()));
        }
        let keys: Vec<_> = request.headers().get("x-api-key").collect();
        match keys.len() {
            0 => Outcome::Failure((Status::BadRequest, ApiKeyError::Missing)),
            1 if is_valid(keys[0]) => Outcome::Success(ApiKey(keys[0].to_string())),
            1 => Outcome::Failure((Status::BadRequest, ApiKeyError::Invalid)),
            _ => Outcome::Failure((Status::BadRequest, ApiKeyError::BadCount)),
        }
    }
}

#[get("/list?<name>&<test>")]
fn bots(
    _key: ApiKey,
    name: Option<String>,
    test: Option<bool>,
) -> String {
}

#[launch]
fn rocket() -> rocket::Rocket {
    rocket::ignite().mount("/", routes![bots, bots_pretty, is_bot])
}
