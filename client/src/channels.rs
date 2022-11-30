use crate::{net, Result};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct Channel(pub String);

// Channels are lowercase when received from the server but should be
// displayed with the first character uppercase when displayed
impl Display for Channel {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", &self.0[..1].to_uppercase(), &self.0[1..])
    }
}

#[derive(Default, Debug, Clone)]
pub struct Channels {
    pub names: Vec<Channel>,
}

impl Channels {
    pub(crate) async fn fetch(url: String) -> Result<Self> {
        let response = net::query(url).await?;

        let names_json = response.json::<Vec<Channel>>().await?;

        Ok(Channels { names: names_json })
    }
}
