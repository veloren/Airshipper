use rocket::{http::Status, *};
use rocket_contrib::json::Json;

use crate::{
    guards::{GitlabEvent, GitlabSecret},
    models::PipelineUpdate,
};

use crate::webhook;

#[post("/", format = "json", data = "<payload>")]
pub async fn post_pipeline_update<'r>(
    _secret: GitlabSecret,
    _event: GitlabEvent,
    payload: Option<Json<PipelineUpdate>>,
    db: crate::DbConnection,
) -> Response<'r> {
    match payload {
        Some(update) => {
            webhook::process(update.clone(), db);
            if update.has_artifacts() {
                Response::build().status(Status::Accepted).finalize()
            } else {
                Response::build().status(Status::Ok).finalize()
            }
        },
        None => Response::build().status(Status::UnprocessableEntity).finalize(),
    }
}
