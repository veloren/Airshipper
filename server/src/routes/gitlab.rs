use rocket::http::Status;
use rocket::*;
use rocket_contrib::json::Json;

use crate::db::DbConnection;
use crate::guards::{GitlabEvent, GitlabSecret};
use crate::models::PipelineUpdate;

use crate::webhook;

#[post("/", format = "json", data = "<payload>")]
pub fn post_pipeline_update<'r>(
    _secret: GitlabSecret,
    _event: GitlabEvent,
    payload: Json<PipelineUpdate>,
    conn: DbConnection,
) -> Response<'r> {
    webhook::process(payload.into_inner(), conn);
    Response::build().status(Status::Ok).finalize()
}
