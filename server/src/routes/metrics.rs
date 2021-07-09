use crate::{metrics::Metrics, Result};
use rocket::*;
use std::sync::Arc;

#[get("/metrics")]
pub fn metrics(metrics: &State<Arc<Metrics>>) -> Result<String> {
    metrics.gather()
}
