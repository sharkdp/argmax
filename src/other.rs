const REASONABLE_DEFAULT_ARG_LENGTH: i64 = 64 * 1024;

pub(crate) fn arg_size<O: AsRef<OsStr>>(arg: O) -> i64 {
    // TODO: this has not been checked and is just a guess.

    size_of::<*const c_char>() as i64 // size for the pointer in argv**
      + arg.as_ref().len() as i64     // size for argument string
      + 1 // terminating NULL
}

/// Total size that is available for arguments to a spawned child process.
pub(crate) fn available_argument_length<O: AsRef<OsStr>>(
    fixed_args: impl Iterator<Item = O>,
) -> Option<i64> {
    Some(REASONABLE_DEFAULT_ARG_LENGTH)
}
