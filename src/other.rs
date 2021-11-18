// TODO: This whole module has not been properly tested. Most of this is just a guess.
// Some of it has been found by trial and error.

use std::ffi::OsStr;

use crate::constants::{POINTER_SIZE_CONSERVATIVE, REASONABLE_DEFAULT_LENGTH};

pub(crate) const MAX_SINGLE_ARGUMENT_LENGTH: i64 = REASONABLE_DEFAULT_LENGTH;

pub(crate) fn arg_size<O: AsRef<OsStr>>(arg: O) -> i64 {
    POINTER_SIZE_CONSERVATIVE // size for the pointer in argv**
      + arg.as_ref().len() as i64     // size for argument string
      + 1 // terminating NULL
}

pub(crate) fn available_argument_length<O: AsRef<OsStr>>(
    fixed_args: impl Iterator<Item = O>,
) -> Option<i64> {
    Some(REASONABLE_DEFAULT_LENGTH - fixed_args.map(|a| arg_size(a.as_ref())).sum::<i64>() - 1)
}

pub(crate) const fn max_single_argument_length() -> i64 {
    MAX_SINGLE_ARGUMENT_LENGTH
}

#[test]
fn show_experimental_limit() {
    use crate::experimental_limit::experimental_arg_limit;

    println!("Experimental limit: {}", experimental_arg_limit());
}
