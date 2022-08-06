// The code in this module is a direct translation of the corresponding C
// implementation in 'bfs' (https://github.com/tavianator/bfs), as of commit
// 9b50adaaaa4fedc8bda6fcf32595ecf7a682fa8b. bfs is licensed under the BSD
// Zero Clause License, copyright Tavian Barnes <tavianator@tavianator.com>.

use std::env;
use std::ffi::OsStr;

use lazy_static::lazy_static;
use nix::unistd::{sysconf, SysconfVar};

use crate::constants::POINTER_SIZE_CONSERVATIVE;

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
    POINTER_SIZE_CONSERVATIVE // size for the pointer in envp**
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
    POINTER_SIZE_CONSERVATIVE // size for the pointer in argv**
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
    arg_max -= POINTER_SIZE_CONSERVATIVE;

    // Account for the arguments so far
    arg_max -= fixed_args.map(|a| arg_size(a.as_ref())).sum::<i64>();
    // Account for the terminating NULL entry
    arg_max -= POINTER_SIZE_CONSERVATIVE;

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
    use crate::experimental_limit::experimental_arg_limit;

    #[test]
    fn available_argument_length_is_smaller_than_experimental_limit() {
        use super::available_argument_length;
        use std::ffi::OsStr;

        let experimental_limit = experimental_arg_limit();
        println!("Experimental limit: {}", experimental_limit);

        let arg_size = 8 + 3 + 1;
        let experimental_size_limit = experimental_limit * arg_size + 8 + 1 + "echo".len() as i64;

        assert!(
            available_argument_length([OsStr::new("echo")].iter()).unwrap_or(0)
                <= experimental_size_limit
        );
    }
}
