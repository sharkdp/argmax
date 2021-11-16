// The code in this module is a direct translation of the corresponding C
// implementation in 'bfs' (https://github.com/tavianator/bfs), as of commit
// 9b50adaaaa4fedc8bda6fcf32595ecf7a682fa8b. bfs is licensed under the BSD
// Zero Clause License, copyright Tavian Barnes <tavianator@tavianator.com>.

use std::env;
use std::ffi::OsStr;
use std::mem::size_of;

use lazy_static::lazy_static;
use libc::c_char;
use nix::unistd::{sysconf, SysconfVar};

const UPPER_BOUND_COMMAND_LINE_LENGTH: i64 = 16 * 1024 * 1024;

lazy_static! {
    static ref PAGE_SIZE: i64 = {
        sysconf(SysconfVar::PAGE_SIZE)
            .ok()
            .flatten()
            .map(|page_size| page_size as i64)
            .filter(|s| *s >= 4096)
            .unwrap_or(4096)
    };

    // TODO: The following is probably Linux specific. See /usr/include/linux/binfmts.h for
    // details.
    pub static ref MAX_SINGLE_ARGUMENT_LENGTH: i64 = 32 * *PAGE_SIZE - 1;
}

/// Required size for a single KEY=VAR environment variable string and the
/// corresponding pointer in envp**.
fn environment_variable_size<O: AsRef<OsStr>>(key: O, value: O) -> i64 {
    size_of::<*const c_char>() as i64 // size for the pointer in envp**
      + key.as_ref().len() as i64     // size for the variable name
      + 1                             // size for the '=' sign
      + value.as_ref().len() as i64   // size for the value
      + 1 // terminating NULL
}

/// Required size to store all environment variables
fn size_of_environment() -> i64 {
    env::vars_os()
        .map(|(key, value)| environment_variable_size(&key, &value))
        .sum()
}

/// Required size to store a single ARG argument and the corresponding
/// pointer in argv**.
pub(crate) fn arg_size<O: AsRef<OsStr>>(arg: O) -> i64 {
    size_of::<*const c_char>() as i64 // size for the pointer in argv**
      + arg.as_ref().len() as i64     // size for argument string
      + 1 // terminating NULL
}

/// Total size that is available for arguments to a spawned child process.
pub(crate) fn available_argument_length<O: AsRef<OsStr>>(
    fixed_args: impl Iterator<Item = O>,
) -> Option<i64> {
    let mut arg_max = sysconf(SysconfVar::ARG_MAX).ok().flatten()? as i64;

    if arg_max < 0 {
        arg_max = UPPER_BOUND_COMMAND_LINE_LENGTH;
    }

    // We have to share space with the environment variables
    arg_max -= size_of_environment();
    // Account for the terminating NULL entry
    arg_max -= size_of::<*const c_char>() as i64;

    // Account for the arguments so far
    arg_max -= fixed_args.map(|a| arg_size(a.as_ref())).sum::<i64>();
    // Account for the terminating NULL entry
    arg_max -= size_of::<*const c_char>() as i64;

    // Assume arguments are counted with the granularity of a single page,
    // so allow a one page cushion to account for rounding up
    arg_max -= *PAGE_SIZE;

    // POSIX recommends an additional 2048 bytes of headroom
    arg_max -= 2048;

    if arg_max < 0 {
        arg_max = 0;
    } else if arg_max > UPPER_BOUND_COMMAND_LINE_LENGTH {
        arg_max = UPPER_BOUND_COMMAND_LINE_LENGTH;
    }

    Some(arg_max)
}

pub(crate) fn max_single_argument_length() -> i64 {
    *MAX_SINGLE_ARGUMENT_LENGTH
}

#[cfg(test)]
mod tests {
    fn command_with_n_args_succeeds(n: i64) -> bool {
        use std::process::Stdio;
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
        use super::available_argument_length;
        use std::ffi::OsStr;

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
