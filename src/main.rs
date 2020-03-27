#[cfg(not(any(feature = "client", feature = "server")))]
compile_error!("Either feature 'client' or 'server' must be enabled for this crate.");
#[cfg(all(feature = "client", feature = "server"))]
compile_error!("Only 'client' or 'server' feature can be used at a time.");

#[cfg(feature = "client")]
fn main() {
    airshipper_client::start();
}

#[cfg(feature = "server")]
fn main() {
    airshipper_server::start();
}
