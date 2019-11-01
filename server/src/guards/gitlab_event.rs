use rocket::http::Status;
use rocket::request::{FromRequest, Outcome, Request};

/// This Request Guard ensures that the event type is "Pipeline Hook"
pub struct GitlabEvent;

#[derive(Debug)]
pub enum GitlabError {
    MissingEvent,
    InvalidEvent,
}

impl<'a, 'r> FromRequest<'a, 'r> for GitlabEvent {
    type Error = GitlabError;

    fn from_request(request: &'a Request<'r>) -> Outcome<Self, Self::Error> {
        let keys: Vec<_> = request.headers().get("X-Gitlab-Event").collect();
        match keys.len() {
            0 => Outcome::Failure((Status::BadRequest, GitlabError::MissingEvent)),
            1 if keys[0] == crate::config::HOOK_TYPE => Outcome::Success(GitlabEvent {}),
            1 => Outcome::Failure((Status::BadRequest, GitlabError::InvalidEvent)),
            _ => Outcome::Failure((Status::BadRequest, GitlabError::InvalidEvent)),
        }
    }
}
