use rocket::{
    http::Status,
    request::{FromRequest, Outcome, Request},
};
use serde::{Deserialize, Serialize};

use crate::conf::Conf;

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenAuth {
    pub owner: String,
}

#[derive(Debug)]
pub struct AuthError;

#[rocket::async_trait]
impl<'r> FromRequest<'r> for TokenAuth {
    type Error = AuthError;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let token = match req.headers().get_one("Authorization") {
            Some(token) => token.to_string(),
            None => return Outcome::Error((Status::Forbidden, AuthError)),
        };

        match req
            .rocket()
            .state::<Conf>()
            .expect("Could not find instane config in rocket state")
            .tokens
            .iter()
            .find_map(|(key, val)| {
                if *val == token {
                    Some(key.to_string())
                } else {
                    None
                }
            }) {
            Some(owner) => Outcome::Success(Self { owner }),
            None => Outcome::Error((Status::Forbidden, AuthError)),
        }
    }
}
