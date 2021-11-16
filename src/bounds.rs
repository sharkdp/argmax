pub(crate) const UPPER_BOUND_ARG_MAX: i64 = 16 * 1024 * 1024;

#[cfg(not(unix))]
pub(crate) const REASONABLE_DEFAULT_ARG_LENGTH: i64 = 64 * 1024;
