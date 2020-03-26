use crate::{
    guards::{GitlabEvent, GitlabSecret},
    models::PipelineUpdate,
    webhook, Result,
};
use rocket::{http::Status, *};
use rocket_contrib::json::Json;

#[tracing::instrument(skip(_secret, _event, payload, db))]
#[post("/", format = "json", data = "<payload>")]
pub async fn post_pipeline_update<'r>(
    _secret: GitlabSecret,
    _event: GitlabEvent,
    payload: Option<Json<PipelineUpdate>>,
    db: crate::DbConnection,
) -> Result<Response<'r>> {
    match payload {
        Some(update) => {
            if let Some(artifacts) = update.artifacts() {
                if db.does_not_exist(&artifacts)? {
                    tracing::debug!("Found {} artifacts.", artifacts.len());
                    webhook::process(artifacts, db);
                    Ok(Response::build().status(Status::Accepted).finalize())
                } else {
                    tracing::warn!("Received duplicate artifacts!");
                    Ok(Response::build().status(Status::BadRequest).finalize())
                }
            } else {
                tracing::debug!("No Artifacts found.");
                Ok(Response::build().status(Status::Ok).finalize())
            }
        },
        None => Ok(Response::build().status(Status::UnprocessableEntity).finalize()),
    }
}
