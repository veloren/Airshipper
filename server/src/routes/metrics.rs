use axum::{extract::State, response::IntoResponse};

use crate::Context;

pub async fn metrics(State(context): State<Context>) -> impl IntoResponse {
    context.metrics.gather()
}
