use std::process::Stdio;

use argmax::Command;

#[test]
fn can_execute_simple_command_with_few_arguments() {
    let mut cmd = Command::new("/bin/echo");
    assert!(cmd.try_arg("foo"));
    assert!(cmd.try_arg("bar"));
    assert_eq!(b"foo bar\n", &cmd.output().unwrap().stdout[..]);
}

#[test]
fn can_run_command_with_maximum_number_of_arguments() {
    let mut try_n_args = 1;
    loop {
        let mut cmd = Command::new("/bin/echo");
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
