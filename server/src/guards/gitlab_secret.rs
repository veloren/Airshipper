use rocket::{
    http::Status,
    request::{FromRequest, Outcome, Request},
};

/// This Request Guard ensures the authenticity of the request is valid
pub struct GitlabSecret;

#[derive(Debug)]
pub enum SecretError {
    MissingSecret,
    InvalidSecret,
    MultipleSecrets,
}

impl<'a, 'r> FromRequest<'a, 'r> for GitlabSecret {
    type Error = SecretError;

    fn from_request(request: &'a Request<'r>) -> Outcome<Self, Self::Error> {
        let keys: Vec<_> = request.headers().get("X-Gitlab-Token").collect();
        match keys.len() {
            0 => Outcome::Failure((Status::Unauthorized, SecretError::MissingSecret)),
            1 if keys[0] == crate::CONFIG.gitlab_secret => Outcome::Success(GitlabSecret {}),
            1 => Outcome::Failure((Status::Unauthorized, SecretError::InvalidSecret)),
            _ => Outcome::Failure((Status::BadRequest, SecretError::MultipleSecrets)),
        }
    }
}
