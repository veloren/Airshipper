use rocket::{
    http::Status,
    request::{FromRequest, Outcome, Request},
};

/// This Request Guard ensures that the event type is "Pipeline Hook"
pub struct GitlabEvent;

#[derive(Debug)]
pub enum GitlabError {
    MissingEvent,
    InvalidEvent,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for GitlabEvent {
    type Error = GitlabError;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let keys: Vec<_> = request.headers().get("X-Gitlab-Event").collect();
        match keys.len() {
            0 => Outcome::Failure((Status::BadRequest, GitlabError::MissingEvent)),
            1 if keys[0] == crate::config::HOOK_TYPE => Outcome::Success(GitlabEvent {}),
            1 => Outcome::Failure((Status::BadRequest, GitlabError::InvalidEvent)),
            _ => Outcome::Failure((Status::BadRequest, GitlabError::InvalidEvent)),
        }
    }
}
