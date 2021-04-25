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
    db: State<'_, crate::DbConnection>,
) -> Result<Response<'r>> {
    match payload {
        Some(update) => {
            if let Some(artifacts) = update.artifacts() {
                if !db.does_not_exist(&artifacts).await? {
                    tracing::warn!("Received duplicate artifacts!");
                }

                tracing::debug!("Found {} artifacts.", artifacts.len());
                webhook::process(artifacts, (*db).clone());
                Ok(Response::build().status(Status::Accepted).finalize())
            } else {
                Ok(Response::build().status(Status::Ok).finalize())
            }
        },
        None => Ok(Response::build()
            .status(Status::UnprocessableEntity)
            .finalize()),
    }
}
