use std::ffi::{OsStr, OsString};
use std::os::windows::ffi::{OsStrExt, OsStringExt};

// Windows C runtime quoting, with surrounding quotes even without whitespace.
// MSYS2/Cygwin otherwise interpret apostrophes and wildcards in bare arguments.
pub(crate) fn quote(arg: &OsStr) -> OsString {
    let mut chars = arg.encode_wide().peekable();
    let mut leading = 0;
    while chars.peek() == Some(&(b'\\' as u16)) {
        leading += 1;
        chars.next();
    }
    // MSYS2 collapses a leading UNC prefix inside quotes. Place doubled
    // leading backslashes before the opening quote; both parsers preserve them.
    let mut quoted = vec![b'\\' as u16; leading * 2];
    quoted.push(b'"' as u16);
    let mut backslashes = 0;
    for ch in chars {
        if ch == b'\\' as u16 {
            backslashes += 1;
            continue;
        }
        let count = if ch == b'"' as u16 {
            backslashes * 2 + 1
        } else {
            backslashes
        };
        quoted.resize(quoted.len() + count, b'\\' as u16);
        quoted.push(ch);
        backslashes = 0;
    }
    quoted.resize(quoted.len() + backslashes * 2, b'\\' as u16);
    quoted.push(b'"' as u16);
    OsString::from_wide(&quoted)
}

pub(crate) fn extra_size(arg: &OsStr) -> i64 {
    // The existing budget includes the original argument's byte length.
    // Retain that conservative estimate when it exceeds the encoded length.
    let mut chars = arg.encode_wide().peekable();
    let mut length = 2;
    while chars.peek() == Some(&(b'\\' as u16)) {
        length += 2;
        chars.next();
    }
    let mut backslashes = 0;
    for ch in chars {
        length += 1;
        if ch == b'\\' as u16 {
            backslashes += 1;
        } else {
            if ch == b'"' as u16 {
                length += backslashes + 1;
            }
            backslashes = 0;
        }
    }
    (length + backslashes - arg.len() as i64).max(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quotes_special_characters_and_trailing_backslashes() {
        for (arg, expected) in [
            ("", "\"\""),
            ("hello'world'", "\"hello'world'\""),
            ("hello world", "\"hello world\""),
            ("a\"b", "\"a\\\"b\""),
            ("a\\", "\"a\\\\\""),
            ("a\\\"b", "\"a\\\\\\\"b\""),
        ] {
            assert_eq!(quote(OsStr::new(arg)), OsStr::new(expected));
        }
    }

    #[test]
    fn preserves_unpaired_surrogates() {
        let arg = OsString::from_wide(&[0xd800, b'\'' as u16]);
        assert_eq!(
            quote(&arg).encode_wide().collect::<Vec<_>>(),
            [34, 0xd800, 39, 34]
        );
    }
}
