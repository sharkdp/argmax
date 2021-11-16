use std::ffi::OsStr;
use std::io;
use std::process::{self, ExitStatus, Output, Stdio};

mod bounds;
#[cfg(unix)]
mod unix;

pub struct Command {
    inner: process::Command,
    remaining_argument_length: i64,
}

impl Command {
    pub fn new<S: AsRef<OsStr>>(program: S) -> Self {
        Command {
            inner: process::Command::new(&program),
            #[cfg(unix)]
            remaining_argument_length: unix::available_argument_length([program].iter())
                .unwrap_or(bounds::UPPER_BOUND_ARG_MAX),
            #[cfg(not(unix))]
            remaining_argument_length: bounds::REASONABLE_DEFAULT_ARG_LENGTH,
        }
    }

    pub fn stdout<T: Into<Stdio>>(&mut self, cfg: T) -> &mut Command {
        self.inner.stdout(cfg);
        self
    }

    pub fn try_arg<S: AsRef<OsStr>>(&mut self, arg: S) -> bool {
        let arg_size = unix::arg_size(&arg);
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
