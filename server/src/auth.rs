use axum::{response::IntoResponse, http::Request, middleware::Next};
use base64::{Engine as _, engine::general_purpose};
use serde::Deserialize;
use anyhow::anyhow;
use crate::{errors::MyError, model::Conf};


pub struct AuthedUser {
  pub email: String
}


pub async fn auth<B>(mut req: Request<B>, next: Next<B>) -> Result<impl IntoResponse, MyError> {

  let conf = Conf::load()?;
  if !conf.require_auth {
    return Ok(next.run(req).await);
  }

  let client_id = "898187078436-49mhvq2bai7te9vjobma6sei8s68iaj9.apps.googleusercontent.com";
  let token = req.headers().get("Authorization")
      .and_then(|h| h.to_str().ok())
      .and_then(|h| h.strip_prefix("Bearer "));

  match token {
    Some(token) => {

      let claims = decode(token)?;
      if claims.aud != client_id {
        tracing::info!("Client id doesn't match: actual: {} vs expected: {}", claims.aud, client_id);
        return Err(MyError::NotAuthorized);
      }

      let user = AuthedUser { email: claims.email.clone() };
      req.extensions_mut().insert(user);

      if !conf.authorized_users.contains(&claims.email) {
        tracing::info!("User is not authorized: {}", claims.email);
        return Err(MyError::NotAuthorized);
      }

      Ok(next.run(req).await)
    },
    None => {
      tracing::info!("No Authorization header or no bearer token in header");
      Err(MyError::NotAuthorized)
    }
  }
}

// WARNING WARNING WARNING
// This is insecure - we should be property validating that the token is signed.
// For now, though, just yank the claim. You can easily fake auth by just crafting
// a JWT with someone elses email address, so this is totally, totally, totally, insecure.
fn decode(token: &str) -> anyhow::Result<Claims> {
  let mut parts = token.split(".");
  let _ = parts.next().ok_or(anyhow!("Invalid token"))?;
  let payload = parts.next().ok_or(anyhow!("Invalid token"))?;
  let _ = parts.next().ok_or(anyhow!("Invalid token"))?;

  let decoded: Claims = serde_json::from_reader(general_purpose::STANDARD_NO_PAD.decode(payload)?.as_slice())?;

  Ok(decoded)
}


#[derive(Deserialize)]
struct Claims {
  email: String,
  aud: String,
  iss: String
}

