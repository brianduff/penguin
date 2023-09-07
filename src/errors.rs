use std::fmt::Display;

use axum::response::IntoResponse;

#[derive(Debug)]
pub enum MyError {
  Failed(anyhow::Error),
  NotFound
}

impl Display for MyError {
    fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::result::Result::Ok(())
    }
}

impl From<anyhow::Error> for MyError {
  fn from(err: anyhow::Error) -> MyError {
      MyError::Failed(err)
  }
}

// Tell axum how to convert `AppError` into a response.
impl IntoResponse for MyError {
    fn into_response(self) -> axum::response::Response {
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {:?}", self),
        )
            .into_response()
    }
}

pub type Result<T> = std::result::Result<T, MyError>;
