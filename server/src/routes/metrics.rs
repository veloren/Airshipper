use crate::{metrics::Metrics, Result};
use rocket::*;
use std::sync::Arc;

#[allow(clippy::result_large_err)]
#[get("/metrics")]
pub fn metrics(metrics: &State<Arc<Metrics>>) -> Result<String> {
    metrics.gather()
}
