use iced::futures;
use std::process::{ExitStatus, Stdio};
use tokio::{
    io::BufReader,
    prelude::*,
    process::Command,
    stream::{Stream, StreamExt},
};

/// Returns a stream of stdout/stderr lines of the Process
pub(crate) fn stream_process(
    cmd: &mut Command,
) -> Result<impl Stream<Item = ProcessUpdate>, tokio::io::Error> {
    // Avoid allocating a console
    #[cfg(windows)]
    cmd.creation_flags(winapi::um::winbase::DETACHED_PROCESS);

    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());
    let mut child = cmd.kill_on_drop(false).spawn()?;

    let stdout = child.stdout.take().unwrap(); // Safe because we setup stdout & stderr beforehand
    let stderr = child.stderr.take().unwrap();

    // Merge stdout and stderr together
    let reader = BufReader::new(stdout)
        .lines()
        .merge(BufReader::new(stderr).lines());
    let exit_status = tokio::spawn(async { child.await });

    Ok(reader
        .map(|x| match x {
            Ok(x) => ProcessUpdate::Line(x),
            Err(e) => ProcessUpdate::Error(e),
        })
        .chain(futures::stream::once(async {
            match exit_status.await {
                Ok(x) => match x {
                    Ok(x) => ProcessUpdate::Exit(x),
                    Err(e) => ProcessUpdate::Error(e),
                },
                Err(e) => ProcessUpdate::Error(e.into()),
            }
        })))
}

#[derive(Debug)]
pub enum ProcessUpdate {
    Line(String),
    Exit(ExitStatus),
    Error(std::io::Error),
}
