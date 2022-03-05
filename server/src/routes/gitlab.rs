use crate::{
    guards::{GitlabEvent, GitlabSecret},
    metrics::Metrics,
    models::PipelineUpdate,
    webhook, Result,
};
use rocket::{http::Status, serde::json::Json, *};
use std::sync::Arc;
use tracing::*;

#[tracing::instrument(skip(_secret, _event, metrics, payload, db))]
#[post("/", format = "json", data = "<payload>")]
pub async fn post_pipeline_update(
    _secret: GitlabSecret,
    _event: GitlabEvent,
    payload: Option<Json<PipelineUpdate>>,
    metrics: &State<Arc<Metrics>>,
    db: crate::DbConnection,
) -> Result<Status> {
    match payload {
        Some(update) => {
            let pipeline_id = update.object_attributes.id;
            match update.channel() {
                Some(channel) => {
                    let artifacts = update.artifacts(&channel);
                    if artifacts.is_empty() {
                        tracing::debug!(
                            ?pipeline_id,
                            ?channel,
                            "Request rejected, no artifacts"
                        );
                        Ok(Status::Ok)
                    } else {
                        let channel = channel.clone();
                        let c = channel.clone();
                        tokio::spawn(
                            webhook::process(artifacts, c, db)
                                .instrument(tracing::info_span!("")),
                        );
                        metrics.uploads.inc();
                        Ok(Status::Accepted)
                    }
                },
                None => {
                    tracing::trace!(?pipeline_id, "Request rejected, no channel");
                    Ok(Status::Ok)
                },
            }
        },
        None => Ok(Status::UnprocessableEntity),
    }
}
