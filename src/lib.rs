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
use std::process::{self, ExitStatus, Output, Stdio};

mod bounds;
#[cfg(not(unix))]
mod other;
#[cfg(unix)]
mod unix;

#[cfg(not(unix))]
use other as platform;
#[cfg(unix)]
use unix as platform;

pub struct Command {
    inner: process::Command,
    remaining_argument_length: i64,
}

impl Command {
    pub fn new<S: AsRef<OsStr>>(program: S) -> Self {
        Command {
            inner: process::Command::new(&program),
            remaining_argument_length: platform::available_argument_length([program].iter())
                .unwrap_or(bounds::REASONABLE_DEFAULT_LENGTH),
        }
    }

    pub fn stdout<T: Into<Stdio>>(&mut self, cfg: T) -> &mut Command {
        self.inner.stdout(cfg);
        self
    }

    pub fn try_arg<S: AsRef<OsStr>>(&mut self, arg: S) -> bool {
        let arg_size = platform::arg_size(&arg);
        if arg_size > self.remaining_argument_length {
            false
        } else {
            self.remaining_argument_length -= arg_size;
            self.inner.arg(arg);
            true
        }
    }

    pub fn output(&mut self) -> io::Result<Output> {
        self.inner.output()
    }

    pub fn status(&mut self) -> io::Result<ExitStatus> {
        self.inner.status()
    }
}
