use crate::models::Platform;
use rocket::http::RawStr;
use rocket::request::FromParam;

impl<'a> FromParam<'a> for Platform {
    type Error = String;

    fn from_param(param: &'a RawStr) -> Result<Self, Self::Error> {
        match param.to_lowercase().as_str() {
            "windows" => Ok(Self::Windows),
            "linux" => Ok(Self::Linux),
            _ => Err(param.to_string()),
        }
    }
}
