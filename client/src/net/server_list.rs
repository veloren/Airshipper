use crate::{Result, query};
use veloren_serverbrowser_api::GameServerList;

pub(crate) async fn fetch_server_list(url: String) -> Result<GameServerList> {
    let response = query(url).await?;

    let server_list = response.json::<GameServerList>().await?;

    Ok(server_list)
}
