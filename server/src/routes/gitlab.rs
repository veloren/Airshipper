use rocket::http::Status;
use rocket::*;
use rocket_contrib::json::Json;

use crate::guards::{GitlabEvent, GitlabSecret};
use crate::models::PipelineUpdate;

use crate::webhook;

#[post("/", format = "json", data = "<payload>")]
pub fn post_pipeline_update<'r>(
    _secret: GitlabSecret,
    _event: GitlabEvent,
    payload: Json<PipelineUpdate>,
    db: State<crate::Database>,
) -> Response<'r> {
    let clone = db.inner().clone().to_owned();
    webhook::process(payload.into_inner(), clone);
    Response::build().status(Status::Ok).finalize()
}
