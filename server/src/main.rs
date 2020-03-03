fn main() {
    dotenv::from_filename(".airshipper-env").ok();
    match server::rocket() {
        Ok(server) => {
            let error = server.launch();
            log::error!("Launch failed with: {}", error);
        },
        Err(e) => log::error!("Failed to start server: {:?}", e),
    };
}
