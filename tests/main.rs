use std::process::Stdio;

use argmax::Command;

#[cfg(not(windows))]
fn get_echo_command() -> Command {
    Command::new("/bin/echo")
}

#[cfg(windows)]
fn get_echo_command() -> Command {
    let mut cmd = Command::new("cmd");
    cmd.try_arg("/c");
    cmd.try_arg("echo");
    cmd
}

#[test]
fn can_execute_simple_command_with_few_arguments() {
    let mut cmd = get_echo_command();

    assert!(cmd.try_arg("foo"));
    assert!(cmd.try_arg("bar"));

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
