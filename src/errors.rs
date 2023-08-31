use std::fmt::Display;

#[derive(Debug)]
pub enum MyError {
  Failed(anyhow::Error),
  NotFound
}

impl Display for MyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::result::Result::Ok(())
    }
}

impl actix_web::error::ResponseError for MyError {
    fn status_code(&self) -> actix_web::http::StatusCode {
        match self {
            MyError::NotFound => actix_web::http::StatusCode::NOT_FOUND,
            _ => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR
        }
    }

    fn error_response(&self) -> actix_web::HttpResponse<actix_web::body::BoxBody> {
        actix_web::HttpResponse::new(self.status_code())
    }
}

impl From<anyhow::Error> for MyError {
  fn from(err: anyhow::Error) -> MyError {
      MyError::Failed(err)
  }
}

pub type Result<T> = std::result::Result<T, MyError>;
