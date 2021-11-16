use std::ffi::OsStr;
use std::io;
use std::process::{self, ExitStatus, Output, Stdio};

mod bounds;
mod unix;

pub struct Command {
    inner: process::Command,
    remaining_argument_length: i64,
}

impl Command {
    pub fn new<S: AsRef<OsStr>>(program: S) -> Self {
        Command {
            inner: process::Command::new(&program),
            remaining_argument_length: unix::available_argument_length([program].iter())
                .unwrap_or(bounds::UPPER_BOUND_ARG_MAX),
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

#[cfg(test)]
mod tests {
    use std::ffi::OsStr;
    use std::process::Stdio;

    use super::unix::available_argument_length;

    fn command_with_n_args_succeeds(n: i64) -> bool {
        std::process::Command::new("/bin/echo")
            .args((0..n).map(|_| "foo"))
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|status| status.success())
            .unwrap_or(false)
    }

    fn binary_search(mut lower: i64, mut upper: i64) -> i64 {
        while lower <= upper {
            let n = (lower + upper) / 2;

            if command_with_n_args_succeeds(n) {
                lower = n + 1;
            } else {
                upper = n - 1;
            }
        }

        lower
    }

    fn experimental_arg_limit() -> i64 {
        binary_search(0, 1_000_000)
    }

    #[test]
    fn available_argument_length_is_smaller_than_experimental_limit() {
        let experimental_limit = experimental_arg_limit();
        println!("Experimental limit: {}", experimental_limit);

        let arg_size = 8 + 3 + 1;
        let experimental_size_limit =
            experimental_limit * arg_size + 8 + 1 + "/bin/echo".len() as i64;

        assert!(
            available_argument_length([OsStr::new("/bin/echo")].iter()).unwrap_or(0)
                <= experimental_size_limit
        );
    }
}
