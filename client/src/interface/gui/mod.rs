use crate::config::ClientConfig;

pub fn run(_config: &mut ClientConfig) {
    log::info!("GUI is currently unsupported. Try without the --gui flag.");
    std::process::exit(0);
}
