/// deals with finding os specific paths and extensions.
pub mod fs;
/// stream process output line by line followed by the exit status
pub mod process;

pub use fs::*;
pub use process::*;
