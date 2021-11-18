pub(crate) const REASONABLE_DEFAULT_LENGTH: i64 = 8 * 1024;

/// We make a conservative guess for the size of a single pointer (64-bit) here
/// in order to support scenarios where a 32-bit binary is launching a 64-bit
/// binary.
pub(crate) const POINTER_SIZE_CONSERVATIVE: i64 = 8;
