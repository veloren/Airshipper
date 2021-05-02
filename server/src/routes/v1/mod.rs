// All gitlab related routes
pub mod gitlab;
// All routes which the user may see
pub mod user;
// exposing api for e.g. querying latest version
pub mod api;
// Catch all case
pub mod catchers;
// Exposes prometheus statistics
pub mod metrics;
