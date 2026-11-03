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
//! let mut cmd = Command::new("echo");
//!
//! // Add as many arguments as possible
//! while cmd.try_arg("foo").is_ok() {}
//!
//! assert!(cmd.status().unwrap().success());
//! # }
//! ```

use std::ffi::OsStr;
#[cfg(windows)]
use std::ffi::OsString;
use std::io;
use std::ops::Deref;
use std::path::Path;
use std::process::{self, Child, ExitStatus, Output, Stdio};

mod constants;
#[cfg(not(unix))]
mod other;
#[cfg(unix)]
mod unix;
#[cfg(windows)]
mod windows;

#[cfg(not(unix))]
use other as platform;
#[cfg(unix)]
use unix as platform;

#[cfg(test)]
mod experimental_limit;

#[derive(Debug)]
pub struct Command {
    inner: process::Command,
    remaining_argument_length: i64,
    #[cfg(windows)]
    quote_args: bool,
    #[cfg(windows)]
    args: Vec<OsString>,
}

impl Command {
    /// See [`std::process::Command::new`][process::Command#method.new].
    pub fn new<S: AsRef<OsStr>>(program: S) -> Self {
        Command {
            inner: process::Command::new(&program),
            remaining_argument_length: platform::available_argument_length([program].iter())
                .unwrap_or(constants::REASONABLE_DEFAULT_LENGTH),
            #[cfg(windows)]
            quote_args: false,
            #[cfg(windows)]
            args: Vec::new(),
        }
    }

    /// Create a command that quotes every Windows argument for MSYS2/Cygwin
    /// compatibility. Arguments remain literal for programs using the C runtime.
    ///
    /// Do not use this constructor for `cmd.exe` or batch files, whose command
    /// lines have different escaping rules. Use [`Command::new`] for those.
    #[cfg(windows)]
    pub fn new_quoted<S: AsRef<OsStr>>(program: S) -> Self {
        let mut command = Self::new(program);
        command.quote_args = true;
        command
    }

    /// Return the original arguments, without Windows command-line escaping.
    #[cfg(windows)]
    pub fn get_args(
        &self,
    ) -> impl DoubleEndedIterator<Item = &OsStr> + ExactSizeIterator + Clone + std::fmt::Debug {
        self.args.iter().map(OsString::as_os_str)
    }

    /// Return the appropriate error for a too-big command line.
    fn e2big() -> io::Error {
        // io::ErrorKind::ArgumentListTooLong is unstable, so get it manually
        #[cfg(unix)]
        return io::Error::from_raw_os_error(libc::E2BIG);
        #[cfg(not(unix))]
        return io::ErrorKind::Other.into();
    }

    /// Get the size of some arguments, if they'll fit.
    fn check_size<I, S>(&self, args: I) -> io::Result<i64>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        let mut size = 0;
        for arg in args {
            let arg = arg.as_ref();
            if arg.len() as i64 > platform::max_single_argument_length() {
                return Err(Self::e2big());
            }
            size += platform::arg_size(arg);
            #[cfg(windows)]
            if self.quote_args {
                // Account for quotes and doubled backslashes as well as UTF-16.
                size += windows::extra_size(arg);
            }
        }

        if size > self.remaining_argument_length {
            return Err(Self::e2big());
        }

        Ok(size)
    }

    /// Check if an additional argument would fit in the command.
    pub fn arg_would_fit<S: AsRef<OsStr>>(&self, arg: S) -> bool {
        self.args_would_fit(&[arg])
    }

    /// Check if multiple additional arguments would all fit in the command.
    pub fn args_would_fit<I, S>(&self, args: I) -> bool
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        self.check_size(args).is_ok()
    }

    /// Like [`std::process::Command::arg`][process::Command#method.arg], add an argument to the
    /// command, but only if it will fit.
    pub fn try_arg<S: AsRef<OsStr>>(&mut self, arg: S) -> io::Result<&mut Self> {
        self.try_args(&[arg])
    }

    /// Like [`std::process::Command::arg`][process::Command#method.args], add multiple arguments to
    /// the command, but only if they will all fit.
    pub fn try_args<I, S>(&mut self, args: I) -> io::Result<&mut Self>
    where
        I: IntoIterator<Item = S> + Copy,
        S: AsRef<OsStr>,
    {
        let size = self.check_size(args)?;
        #[cfg(not(windows))]
        self.inner.args(args);
        #[cfg(windows)]
        for arg in args {
            let arg = arg.as_ref();
            if self.quote_args {
                use std::os::windows::process::CommandExt;
                self.inner.raw_arg(windows::quote(arg));
            } else {
                self.inner.arg(arg);
            }
            self.args.push(arg.to_owned());
        }
        self.remaining_argument_length -= size;
        Ok(self)
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
    pub fn spawn(&mut self) -> io::Result<Child> {
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

impl Deref for Command {
    type Target = process::Command;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(windows)]
    #[test]
    fn quoted_arguments_respect_budget_and_failed_additions_are_atomic() {
        let mut command = Command::new_quoted("echo");
        command.remaining_argument_length = 15;
        // A trailing backslash needs doubling plus two surrounding quotes.
        assert!(!command.arg_would_fit("\\\\\\"));
        assert!(command.try_args(["ok", "\\\\\\"]).is_err());
        assert_eq!(command.get_args().len(), 0);
        assert_eq!(command.inner.get_args().len(), 0);
        assert_eq!(command.remaining_argument_length, 15);
        command.try_arg("ok").unwrap();
        assert_eq!(command.get_args().collect::<Vec<_>>(), [OsStr::new("ok")]);
    }

    #[test]
    fn test_api() {
        let mut cmd = Command::new("echo");
        cmd.try_arg("Hello").expect("try_arg() to succeed");
        cmd.try_args(&["world", "!"])
            .expect("try_args() to succeed");

        cmd.current_dir(".");

        cmd.stdin(Stdio::inherit());
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::null());

        let mut output = cmd
            .spawn()
            .expect("spawn() to succeed")
            .wait_with_output()
            .expect("wait_with_output() to succeed");
        assert!(output.stdout.len() > 13);
        assert!(output.status.success());

        output = cmd.output().expect("output() to succeed");
        assert!(output.stdout.len() > 13);
        assert!(output.status.success());

        let status = cmd.status().expect("status() to succeed");
        assert!(status.success());
    }
}
