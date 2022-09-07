use crate::{net, Result};
use country_parser::Country;
use serde::{
    de::{Deserializer, Error, Unexpected},
    Deserialize,
};

#[derive(Deserialize, Debug, Clone)]
pub struct ServerList {
    pub servers: Vec<Server>,
}

impl ServerList {
    pub(crate) async fn fetch(url: String) -> Result<Self> {
        let response = net::query(url).await?;

        let server_list = response.json::<ServerList>().await?;

        Ok(server_list)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Server {
    /// The name of the server.
    pub name: String,
    /// The server description.
    pub description: String,
    /// The address through which the server might be accessed on the open internet. This
    /// field may be an IPv4 address, IPv6 address, URL, and may or may not contain a
    /// port (14004 is assumed if unspecified).
    pub address: String,
    /// The country that the server is physically based in (note: this field is intended
    /// as an indication of factors like ping, not the language of the server).
    #[serde(deserialize_with = "deserialize_country")]
    #[serde(default)]
    pub location: Option<Country>,
    /// The auth server that must be used to connect to this server. `None` means the
    /// official auth server.
    #[serde(default, rename = "authServer")]
    pub auth_server: Option<String>,
    /// The version channel used by the server. `None` means not running a channel
    /// distributed by Airshipper. If in doubt, `"weekly"` is probably correct.
    #[serde(default)]
    pub channel: Option<String>,
    /// Whether the server is officially affiliated with the Veloren project.
    pub official: bool,
}

fn deserialize_country<'de, D: Deserializer<'de>>(
    de: D,
) -> std::result::Result<Option<Country>, D::Error> {
    let res = String::deserialize(de);
    res.map_or(Ok(None), |x| {
        country_parser::parse(x).map(Some).ok_or_else(|| {
            D::Error::invalid_value(
                Unexpected::Other("invalid country"),
                &"valid ISO-3166 country",
            )
        })
    })
}
