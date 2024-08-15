#![allow(unused_imports)]
use axum::{extract::State, response::IntoResponse};

use crate::{metrics::Metrics, Context, Result};
use std::sync::Arc;

pub async fn metrics(State(context): State<Context>) -> impl IntoResponse {
    context.metrics.gather()
}
