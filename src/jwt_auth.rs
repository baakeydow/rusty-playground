extern crate jsonwebtoken as jwt;
use actix_web::guard::GuardContext;
use rusty_lib::dtkutils::dtk_error::DtkError;
use rusty_lib::dtkutils::dtk_reqwest::{get_token_info, TokenInfo};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct AuthData {
    header: serde_json::Value,
    pub claims: TokenInfo,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct JwtAuth {
    pub claims: TokenInfo,
    pub token: String,
}

impl JwtAuth {
    pub fn new(ctx: &GuardContext) -> Result<Self, DtkError> {
        let user_id = match ctx.head().headers().get("user_id") {
            Some(user_id) => user_id,
            None => return Err(DtkError::from("No user_id found in request header")),
        };
        let auth_header = match ctx.head().headers().get("Authorization") {
            Some(auth_header) => auth_header,
            None => return Err(DtkError::from("No Authorization header found")),
        };
        let auth_value = auth_header.to_str().map_err(DtkError::from).unwrap();
        let token = auth_value.replace("Bearer ", "");
        let auth_data = get_token_info(token.clone(), user_id.clone().to_str().unwrap().to_string()).unwrap();
        let jwt = JwtAuth {
            claims: auth_data,
            token,
        };
        if jwt.claims.id != user_id.to_str().unwrap() {
            return Err(DtkError::from("JWT not valid"));
        }
        log::info!("auth_data: {:?}", &jwt);
        Ok(jwt)
    }
}
