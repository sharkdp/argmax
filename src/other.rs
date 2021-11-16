// TODO: This whole module has not been properly tested. Most of this is just a guess.
// Some of it has been found by trial and error.

use std::ffi::OsStr;
use std::mem::size_of;

use libc::c_char;

const REASONABLE_DEFAULT_ARG_LENGTH: i64 = 16 * 1024;

pub(crate) fn arg_size<O: AsRef<OsStr>>(arg: O) -> i64 {
    size_of::<*const c_char>() as i64 // size for the pointer in argv**
      + arg.as_ref().len() as i64     // size for argument string
      + 1 // terminating NULL
}

/// Total size that is available for arguments to a spawned child process.
pub(crate) fn available_argument_length<O: AsRef<OsStr>>(
    fixed_args: impl Iterator<Item = O>,
) -> Option<i64> {
    Some(REASONABLE_DEFAULT_ARG_LENGTH - fixed_args.map(|a| arg_size(a.as_ref())).sum::<i64>())
}
