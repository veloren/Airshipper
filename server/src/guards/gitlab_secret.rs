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

#[rocket::async_trait]
impl<'r> FromRequest<'r> for GitlabSecret {
    type Error = SecretError;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let keys: Vec<_> = request.headers().get("X-Gitlab-Token").collect();
        match keys.len() {
            0 => Outcome::Failure((Status::Unauthorized, SecretError::MissingSecret)),
            1 => {
                for channel in crate::CONFIG.channels.values() {
                    if channel.gitlab_secret == keys[0] {
                        return Outcome::Success(GitlabSecret {});
                    }
                }
                Outcome::Failure((Status::Unauthorized, SecretError::InvalidSecret))
            },
            _ => Outcome::Failure((Status::BadRequest, SecretError::MultipleSecrets)),
        }
    }
}
