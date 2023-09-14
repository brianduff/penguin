use std::fmt::Display;

use axum::response::IntoResponse;


#[derive(Debug)]
pub enum MyError {
  Failed(anyhow::Error),
  NotFound,
  BadRequest(String)
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
        match self {
            MyError::BadRequest(m) => {
                tracing::error!("Bad request: {:?}", m);
                (
                    axum::http::StatusCode::BAD_REQUEST,
                    m.to_owned(),
                )
            },
            MyError::Failed(e) => {
                tracing::error!("Internal error: {:?}", e);
                (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    format!("An internal error occurred: {:?}", e),
                )
            },
            MyError::NotFound => {
                tracing::error!("Error: not found");
                (
                    axum::http::StatusCode::NOT_FOUND,
                    "".to_owned()
                )

            }
        }.into_response()
    }
}

pub type Result<T> = std::result::Result<T, MyError>;
