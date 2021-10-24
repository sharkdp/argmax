use std::ffi::OsStr;
use std::process::Stdio;

use argmax::detail::available_argument_length;
use argmax::Command;

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
    let experimental_size_limit = experimental_limit * arg_size + 8 + 1 + "/bin/echo".len() as i64;

    assert!(
        available_argument_length([OsStr::new("/bin/echo")].iter()).unwrap_or(0)
            <= experimental_size_limit
    );
}

#[test]
fn test_command_basic() {
    let mut cmd = Command::new("/usr/bin/echo");
    assert!(cmd.try_arg("foo"));
    assert!(cmd.try_arg("bar"));
    assert_eq!(b"foo bar\n", &cmd.output().unwrap().stdout[..]);
}

#[test]
fn test_command_limit() {
    let mut try_n_args = 1;
    loop {
        let mut cmd = Command::new("/usr/bin/echo");
        cmd.stdout(Stdio::null());
        let mut reached_limit = false;

        let mut actual_n_args = 0;
        for _ in 0..try_n_args {
            if !cmd.try_arg("foo") {
                reached_limit = true;
                break;
            }
            actual_n_args += 1;
        }
        println!("Trying execution with {} args", actual_n_args);
        assert!(cmd.status().unwrap().success());

        if reached_limit {
            break;
        }

        try_n_args *= 2;
    }
}
