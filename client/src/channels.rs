use crate::{net, Result};

pub type Channel = String;

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
