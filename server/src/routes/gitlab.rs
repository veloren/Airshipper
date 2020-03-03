use rocket::{http::Status, *};
use rocket_contrib::json::Json;

use crate::{
    guards::{GitlabEvent, GitlabSecret},
    models::PipelineUpdate,
};

use crate::webhook;

#[post("/", format = "json", data = "<payload>")]
pub fn post_pipeline_update<'r>(_secret: GitlabSecret, _event: GitlabEvent, payload: Json<PipelineUpdate>, db: crate::DbConnection) -> Response<'r> {
    webhook::process(payload.into_inner(), db);
    Response::build().status(Status::Ok).finalize()
}
