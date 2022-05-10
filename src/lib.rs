//! `argmax` is a library that allows applications to avoid *Argument list too long* errors
//! (`E2BIG`) by providing a wrapper around [`std::process::Command`]. The [`Command::try_arg`]
//! function can be used to add as many arguments as possible. The function returns `false` if no
//! more arguments can be added (command execution would result in immediate failure).
//!
//! # Usage
//! ```
//! # #[cfg(unix)]
//! # {
//! use argmax::Command;
//!
//! let mut cmd = Command::new("/bin/echo");
//!
//! // Add as many arguments as possible
//! while cmd.try_arg("foo") {}
//!
//! assert!(cmd.status().unwrap().success());
//! # }
//! ```

use std::ffi::OsStr;
use std::io;
use std::path::Path;
use std::process::{self, Child, ExitStatus, Output, Stdio};

mod constants;
#[cfg(not(unix))]
mod other;
#[cfg(unix)]
mod unix;

#[cfg(not(unix))]
use other as platform;
#[cfg(unix)]
use unix as platform;

#[cfg(test)]
mod experimental_limit;

pub struct Command {
    inner: process::Command,
    remaining_argument_length: i64,
}

impl Command {
    /// See [`std::process::Command::new`][process::Command#method.new].
    pub fn new<S: AsRef<OsStr>>(program: S) -> Self {
        Command {
            inner: process::Command::new(&program),
            remaining_argument_length: platform::available_argument_length([program].iter())
                .unwrap_or(constants::REASONABLE_DEFAULT_LENGTH),
        }
    }

    /// Like [`std::process::Command::arg`][process::Command#method.arg], add an argument to the
    /// command, but only if it will fit.
    pub fn try_arg<S: AsRef<OsStr>>(&mut self, arg: S) -> bool {
        if arg.as_ref().len() as i64 > platform::max_single_argument_length() {
            return false;
        }

        let arg_size = platform::arg_size(&arg);
        if arg_size > self.remaining_argument_length {
            false
        } else {
            self.remaining_argument_length -= arg_size;
            self.inner.arg(arg);
            true
        }
    }

    /// See [`std::process::Command::current_dir`][process::Command#method.current_dir].
    pub fn current_dir<P: AsRef<Path>>(&mut self, dir: P) -> &mut Self {
        self.inner.current_dir(dir);
        self
    }

    /// See [`std::process::Command::stdin`][process::Command#method.stdin].
    pub fn stdin<T: Into<Stdio>>(&mut self, cfg: T) -> &mut Self {
        self.inner.stdin(cfg);
        self
    }

    /// See [`std::process::Command::stdout`][process::Command#method.stdout].
    pub fn stdout<T: Into<Stdio>>(&mut self, cfg: T) -> &mut Self {
        self.inner.stdout(cfg);
        self
    }

    /// See [`std::process::Command::stderr`][process::Command#method.stderr].
    pub fn stderr<T: Into<Stdio>>(&mut self, cfg: T) -> &mut Self {
        self.inner.stderr(cfg);
        self
    }

    /// See [`std::process::Command::spawn`][process::Command#method.spawn].
    pub fn spawn<T: Into<Stdio>>(&mut self) -> io::Result<Child> {
        self.inner.spawn()
    }

    /// See [`std::process::Command::output`][process::Command#method.output].
    pub fn output(&mut self) -> io::Result<Output> {
        self.inner.output()
    }

    /// See [`std::process::Command::status`][process::Command#method.status].
    pub fn status(&mut self) -> io::Result<ExitStatus> {
        self.inner.status()
    }
}
