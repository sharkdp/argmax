// The code in this module is a direct translation of the corresponding C
// implementation in 'bfs' (https://github.com/tavianator/bfs), as of commit
// 9b50adaaaa4fedc8bda6fcf32595ecf7a682fa8b. bfs is licensed under the BSD
// Zero Clause License, copyright Tavian Barnes <tavianator@tavianator.com>.

use std::env;
use std::ffi::OsStr;
use std::mem::size_of;

use libc::c_char;
use nix::unistd::{sysconf, SysconfVar};

pub const UPPER_BOUND_ARG_MAX: i64 = 16 * 1024 * 1024;

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
pub fn available_argument_length<O: AsRef<OsStr>>(
    fixed_args: impl Iterator<Item = O>,
) -> Option<i64> {
    let mut arg_max = sysconf(SysconfVar::ARG_MAX).ok().flatten()?;

    if arg_max < 0 {
        arg_max = UPPER_BOUND_ARG_MAX;
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
    let page_size = sysconf(SysconfVar::PAGE_SIZE)
        .ok()
        .flatten()
        .filter(|s| *s >= 4096)
        .unwrap_or(4096);
    arg_max -= page_size;

    // POSIX recommends an additional 2048 bytes of headroom
    arg_max -= 2048;

    if arg_max < 0 {
        arg_max = 0;
    } else if arg_max > UPPER_BOUND_ARG_MAX {
        arg_max = UPPER_BOUND_ARG_MAX;
    }

    Some(arg_max)
}

#[test]
fn test_arg_size() {
    assert_eq!(arg_size(OsStr::new("A")), 10);
}

#[test]
fn test_environment_variable_size() {
    assert_eq!(environment_variable_size("SHELL", "/usr/bin/zsh"), 27);
}
