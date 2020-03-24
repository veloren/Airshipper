use rocket::{http::Status, *};
use rocket_contrib::json::Json;

use crate::{
    guards::{GitlabEvent, GitlabSecret},
    models::PipelineUpdate,
};

use crate::webhook;

#[tracing::instrument(skip(db, _secret, _event, payload))]
#[post("/", format = "json", data = "<payload>")]
pub async fn post_pipeline_update<'r>(
    _secret: GitlabSecret,
    _event: GitlabEvent,
    payload: Option<Json<PipelineUpdate>>,
    db: crate::DbConnection,
) -> Response<'r> {
    match payload {
        Some(update) => {
            if let Some(artifacts) = update.artifacts() {
                tracing::debug!("Found {} artifacts.", artifacts.len());
                webhook::process(artifacts, db);
                Response::build().status(Status::Accepted).finalize()
            } else {
                tracing::debug!("No Artifacts found.");
                Response::build().status(Status::Ok).finalize()
            }
        },
        None => Response::build().status(Status::UnprocessableEntity).finalize(),
    }
}
