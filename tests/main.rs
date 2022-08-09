use std::process::Stdio;

use argmax::Command;

#[cfg(not(windows))]
fn get_echo_command() -> Command {
    Command::new("echo")
}

#[cfg(windows)]
fn get_echo_command() -> Command {
    let mut cmd = Command::new("cmd");
    cmd.try_args(&["/c", "echo"]).expect("try_args(/c echo)");
    cmd
}

#[test]
fn can_execute_simple_command_with_few_arguments() {
    let mut cmd = get_echo_command();

    cmd.try_arg("foo").expect("try_arg(foo)");
    cmd.try_arg("bar").expect("try_arg(bar)");

    #[cfg(not(windows))]
    assert_eq!(b"foo bar\n", &cmd.output().unwrap().stdout[..]);

    #[cfg(windows)]
    assert_eq!(b"foo bar\r\n", &cmd.output().unwrap().stdout[..]);
}

#[test]
fn can_run_command_with_maximum_number_of_arguments() {
    let mut try_n_args = 1;
    loop {
        let mut cmd = get_echo_command();
        cmd.stdout(Stdio::null());
        let mut reached_limit = false;

        let mut actual_n_args = 0;
        for _ in 0..try_n_args {
            if cmd.try_arg("foo").is_err() {
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

#[test]
fn can_run_command_with_single_long_argument() {
    let mut length = 1;
    loop {
        let mut cmd = get_echo_command();
        cmd.stdout(Stdio::null());

        let arg = "x".repeat(length);
        if cmd.try_arg(arg).is_err() {
            break;
        }
        println!("Trying execution with argument length {}", length);
        assert!(cmd.status().unwrap().success());

        length *= 2;
    }
}
