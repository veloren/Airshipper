use iced::futures;
use std::{
    collections::HashMap,
    ffi::OsString,
    path::PathBuf,
    process::{ExitStatus, Stdio},
};
use tokio::{
    io::BufReader,
    prelude::*,
    process::Command,
    stream::{Stream, StreamExt},
};

/// Returns a stream of stdout/stderr lines of the Process
pub(crate) fn stream_process(cmd: CommandBuilder) -> impl Stream<Item = ProcessUpdate> {
    let mut cmd = cmd.build();

    // Avoid allocating a console
    #[cfg(windows)]
    cmd.creation_flags(winapi::um::winbase::DETACHED_PROCESS);

    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());
    let mut child = cmd.kill_on_drop(true).spawn().unwrap();

    let stdout = child.stdout.take().unwrap(); // Safe because we setup stdout & stderr beforehand
    let stderr = child.stderr.take().unwrap();

    // Merge stdout and stderr together
    let reader = BufReader::new(stdout)
        .lines()
        .merge(BufReader::new(stderr).lines());
    let exit_status = tokio::spawn(async { child.await });

    reader
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
        }))
}

#[derive(Debug)]
pub enum ProcessUpdate {
    Line(String),
    Exit(ExitStatus),
    Error(std::io::Error),
}

/// Cloneable alternative to tokio/std::Command
///
/// Very limited and only meant to be cloned and then converted.
/// Credits to https://docs.rs/tokio/0.2.21/src/tokio/process/mod.rs.html
#[derive(Default, Debug, Clone)]
pub struct CommandBuilder {
    program: OsString,
    args: Vec<OsString>,
    envs: HashMap<OsString, OsString>,
    env_remove: Vec<OsString>,
    env_clear: bool,
    current_dir: Option<PathBuf>,
    kill_on_drop: bool,
}

impl CommandBuilder {
    pub fn new<S: Into<OsString>>(program: S) -> Self {
        Self {
            program: program.into(),
            ..Default::default()
        }
    }

    /// Adds an argument to pass to the program.
    pub fn arg<S: Into<OsString>>(&mut self, arg: S) -> &mut Self {
        self.args.push(arg.into());
        self
    }

    /// Adds multiple arguments to pass to the program.
    pub fn args<I, S>(&mut self, args: I) -> &mut Self
    where
        I: IntoIterator<Item = S>,
        S: Into<OsString>,
    {
        args.into_iter()
            .map(|arg| arg.into())
            .for_each(|arg| self.args.push(arg));
        self
    }

    /// Inserts or updates an environment variable mapping.
    ///
    /// Note that environment variable names are case-insensitive (but case-preserving) on
    /// Windows, and case-sensitive on all other platforms.
    pub fn env<S, T>(&mut self, key: S, val: T) -> &mut Self
    where
        S: Into<OsString>,
        T: Into<OsString>,
    {
        self.envs.insert(key.into(), val.into());
        self
    }

    /// Adds or updates multiple environment variable mappings.
    pub fn envs<I, S, T>(&mut self, vars: I) -> &mut Self
    where
        I: IntoIterator<Item = (S, T)>,
        S: Into<OsString>,
        T: Into<OsString>,
    {
        vars.into_iter()
            .map(|(key, val)| (key.into(), val.into()))
            .for_each(|(key, val)| {
                self.envs.insert(key, val);
            });
        self
    }

    /// Removes an environment variable mapping.
    pub fn env_remove<K: Into<OsString>>(&mut self, key: K) -> &mut Self {
        self.env_remove.push(key.into());
        self
    }

    /// Clears the entire environment map for the child process.
    pub fn env_clear(&mut self) -> &mut Self {
        self.env_clear = true;
        self
    }

    /// Sets the working directory for the child process.
    ///
    /// # Platform-specific behavior
    ///
    /// If the program path is relative (e.g., `"./script.sh"`), it's ambiguous
    /// whether it should be interpreted relative to the parent's working
    /// directory or relative to `current_dir`. The behavior in this case is
    /// platform specific and unstable, and it's recommended to use
    /// [`canonicalize`] to get an absolute program path instead.
    ///
    /// [`canonicalize`]: crate::fs::canonicalize()
    pub fn current_dir<P: Into<PathBuf>>(&mut self, dir: P) -> &mut Self {
        self.current_dir = Some(dir.into());
        self
    }

    /// Controls whether a `kill` operation should be invoked on a spawned child
    /// process when its corresponding `Child` handle is dropped.
    ///
    /// By default, this value is assumed to be `false`, meaning the next spawned
    /// process will not be killed on drop, similar to the behavior of the standard
    /// library.
    pub fn kill_on_drop(&mut self, kill_on_drop: bool) -> &mut Self {
        self.kill_on_drop = kill_on_drop;
        self
    }

    /// Build tokio::process::Command ready to be consumed.
    pub fn build(&self) -> Command {
        let mut cmd = Command::new(&self.program);
        cmd.args(&self.args);
        cmd.envs(&self.envs);
        self.env_remove.iter().for_each(|s| {
            cmd.env_remove(s);
        });
        if self.env_clear {
            cmd.env_clear();
        }
        if let Some(dir) = &self.current_dir {
            cmd.current_dir(dir);
        }
        cmd.kill_on_drop(self.kill_on_drop);
        cmd
    }
}
