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
        Some(mut update) => {
            let pipeline_id = update.object_attributes.id;
            let _span = span!(Level::INFO, "", ?pipeline_id);
            if !update.early_filter() {
                tracing::trace!("early return");
                return Ok(Status::Ok);
            }
            //Extend payload with variables
            if let Err(e) = update.extends_variables().await {
                tracing::warn!(?e, "couldn't extend variables");
            };
            tracing::info!(?update.object_attributes.variables, "got variables");

            match update.channel() {
                Some(channel) => {
                    let artifacts = update.artifacts(&channel);
                    if artifacts.is_empty() {
                        tracing::debug!(?channel, "Request rejected, no artifacts");
                        Ok(Status::Ok)
                    } else {
                        for a in &artifacts {
                            metrics.increment_artifact_upload(&a.os, &a.arch, &channel);
                        }
                        let c = channel.clone();
                        tokio::spawn(
                            webhook::process(artifacts, channel, db)
                                .instrument(tracing::info_span!("")),
                        );
                        metrics.increment_upload(&c);
                        Ok(Status::Accepted)
                    }
                },
                None => {
                    tracing::trace!("Request rejected, no channel");
                    Ok(Status::Ok)
                },
            }
        },
        None => Ok(Status::UnprocessableEntity),
    }
}
