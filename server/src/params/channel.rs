use crate::models::Channel;
use rocket::{http::RawStr, request::FromParam};

impl<'a> FromParam<'a> for Channel {
    type Error = String;

    fn from_param(param: &'a RawStr) -> Result<Self, Self::Error> {
        match param.to_lowercase().as_str() {
            "nightly" => Ok(Self::Nightly),
            _ => Err(param.to_string()),
        }
    }
}
