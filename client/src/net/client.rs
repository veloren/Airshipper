use crate::Result;
use reqwest::IntoUrl;

// Name your user agent after your app?
const USER_AGENT: &str = concat!("Airshipper/", env!("CARGO_PKG_VERSION"));

lazy_static::lazy_static! {
    // Base for config, profiles, ...
    pub static ref WEB_CLIENT: reqwest::Client = {
        reqwest::Client::builder()
            .user_agent(USER_AGENT)
            .use_rustls_tls()
            .connect_timeout(std::time::Duration::from_secs(10))
            .build()
            .expect("FATAL: Failed to build reqwest client!")
    };
}

/// Queries url for the etag header
/// Note: Will default to `MISSING_ETAG` incase header isn't found
pub(crate) async fn query_etag<U: IntoUrl>(url: U) -> Result<Option<String>> {
    Ok(WEB_CLIENT
        .head(url)
        .send()
        .await?
        .headers()
        .get("etag")
        .and_then(|s| s.to_str().map(String::from).ok()))
}

/// Extracts Etag value from response
/// Note: Will default to `MISSING_ETAG` incase header isn't found
pub(crate) fn get_etag(x: &reqwest::Response) -> String {
    x.headers().get("etag").map(|x| x.to_str().unwrap().to_string()) // Etag will always be a valid UTF-8 due to it being ASCII
        .unwrap_or_else(|| "MISSING_ETAG".into())
}

pub(crate) async fn query<U: IntoUrl>(url: U) -> Result<reqwest::Response> {
    Ok(WEB_CLIENT.get(url).send().await?)
}
