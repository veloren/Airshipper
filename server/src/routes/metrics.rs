use crate::{metrics::Metrics, Result};
use rocket::*;

#[get("/metrics")]
pub fn metrics(metrics: &State<Metrics>) -> Result<String> {
    metrics.gather()
}
