use rocket::{http::Status, response::Responder, Request, Response};
use thiserror::Error;

use std::io::Cursor;

#[derive(Error, Debug)]
pub enum ProvingServerError {
    #[error("HTTP Error {source:?}")]
    Reqwest {
        #[from]
        source: reqwest::Error,
    },
    #[error("Bad Proof Input Error: {message}")]
    BadProofInputsError { message: String },
}

impl<'r, 'o: 'r> Responder<'r, 'o> for ProvingServerError {
    fn respond_to(self, req: &'r Request<'_>) -> rocket::response::Result<'o> {
        // log `self` to your favored error tracker, e.g.
        // sentry::capture_error(&self);
        match self {
            ProvingServerError::BadProofInputsError { message } => Response::build()
                .sized_body(
                    format!("{} is missing from your inputs", message).len(),
                    Cursor::new(format!("{} is missing from your inputs", message)),
                )
                .status(Status::BadRequest)
                .ok(),
            // in our simplistic example, we're happy to respond with the default 500 responder in all cases
            _ => Status::InternalServerError.respond_to(req),
        }
    }
}
