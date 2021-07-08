use crate::{
    guards::{GitlabEvent, GitlabSecret},
    models::PipelineUpdate,
    webhook, Result,
};
use rocket::{http::Status, serde::json::Json, *};

#[tracing::instrument(skip(_secret, _event, payload, db))]
#[post("/", format = "json", data = "<payload>")]
pub async fn post_pipeline_update(
    _secret: GitlabSecret,
    _event: GitlabEvent,
    payload: Option<Json<PipelineUpdate>>,
    db: crate::DbConnection,
) -> Result<Status> {
    match payload {
        Some(update) => {
            if let Some(artifacts) = update.artifacts() {
                if !db.does_not_exist(&artifacts).await? {
                    tracing::warn!("Received duplicate artifacts!");
                }

                tracing::debug!("Found {} artifacts.", artifacts.len());
                webhook::process(artifacts, db);
                Ok(Status::Accepted)
            } else {
                Ok(Status::Ok)
            }
        },
        None => Ok(Status::UnprocessableEntity),
    }
}
