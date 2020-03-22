// How to send manual webhooks:
// curl --header "Content-Type: application/json" --request POST --data "@<FILE_WITH_WEBHOOK_DATA>" --header "X-Gitlab-Event: Pipeline Hook" --header "X-Gitlab-Token: <TOKEN>" http://<ADDRESS>

fn main() {
    dotenv::from_filename(".airshipper-env").ok();
    match server::rocket() {
        Ok(server) => {
            let error = server.launch();
            log::error!("Launch failed with: {:#?}", error);
        },
        Err(e) => log::error!("Failed to start server: {:?}", e),
    };
}
