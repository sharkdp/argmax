use std::ffi::OsStr;
use std::io;
use std::process::{self, ExitStatus, Output, Stdio};

pub mod detail;

pub struct Command {
    inner: process::Command,
    remaining_argument_length: i64,
}

impl Command {
    pub fn new<S: AsRef<OsStr>>(program: S) -> Self {
        Command {
            inner: process::Command::new(&program),
            remaining_argument_length: detail::available_argument_length([program].iter())
                .unwrap_or(detail::UPPER_BOUND_ARG_MAX),
        }
    }

    pub fn stdout<T: Into<Stdio>>(&mut self, cfg: T) -> &mut Command {
        self.inner.stdout(cfg);
        self
    }

    pub fn try_arg<S: AsRef<OsStr>>(&mut self, arg: S) -> bool {
        let arg_size = detail::arg_size(&arg);
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
