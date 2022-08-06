use std::process::Command;

#[cfg(not(windows))]
fn get_echo_command() -> Command {
    Command::new("echo")
}

#[cfg(windows)]
fn get_echo_command() -> Command {
    let mut cmd = Command::new("cmd");
    cmd.arg("/c");
    cmd.arg("echo");
    cmd
}

fn command_with_n_args_succeeds(n: i64) -> bool {
    use std::process::Stdio;
    get_echo_command()
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

pub(crate) fn experimental_arg_limit() -> i64 {
    std::env::set_var("WINEDEBUG", "-all");

    binary_search(0, 1_000_000)
}
