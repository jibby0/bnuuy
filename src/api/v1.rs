pub mod dog;
use crate::StdResult;

use rocket::{
    http::Status,
    request::Request,
    response::{Responder, Response},
};

#[derive(Debug)]
pub struct RespErr(Status);
type Resp<T> = StdResult<T, RespErr>;

impl<'r> Responder<'r> for RespErr {
    fn respond_to(self, req: &Request) -> StdResult<Response<'r>, Status> {
        self.0.respond_to(req)
    }
}

/// Allow error handling with `?` for 500 errors.
impl From<diesel::result::Error> for RespErr {
    fn from(error: diesel::result::Error) -> Self {
        log::error!("{}", error);
        RespErr(Status::ServiceUnavailable)
    }
}

impl From<std::io::Error> for RespErr {
    fn from(error: std::io::Error) -> Self {
        log::error!("{}", error);
        RespErr(Status::ServiceUnavailable)
    }
}
