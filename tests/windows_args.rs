#[cfg(windows)]
fn main() {
    use std::ffi::OsString;
    use std::io::Write;
    use std::os::windows::ffi::OsStrExt;

    fn encode(args: impl Iterator<Item = OsString>) -> Vec<u8> {
        let mut bytes = Vec::new();
        for arg in args {
            let wide: Vec<_> = arg.encode_wide().collect();
            bytes.extend((wide.len() as u32).to_le_bytes());
            for unit in wide {
                bytes.extend(unit.to_le_bytes());
            }
        }
        bytes
    }

    // Reuse this native executable as a child that reports precisely which
    // UTF-16 arguments Windows delivered, without shell interpretation.
    if std::env::args_os().nth(1).as_deref() == Some(std::ffi::OsStr::new("--echo")) {
        std::io::stdout()
            .write_all(&encode(std::env::args_os().skip(2)))
            .unwrap();
        return;
    }
    let fixtures = [
        "",
        "hello'world'",
        "hello 'world'",
        "hello\\'world\\'",
        "a'b\\",
        "a'b\"c",
        "\\\\server\\share\\",
        "\\\\server\\share'quoted\\",
        "\\",
        "\\\\",
        "\\\\\\leading",
        "a\\\\\"b",
        "*?[abc]",
        "雪'😀",
        "line\nbreak",
    ];
    let mut combinations = Vec::new();
    for a in ["", "a", "'", "\"", "\\", " ", "\t"] {
        for b in ["", "a", "'", "\"", "\\", " ", "\t"] {
            for c in ["", "a", "'", "\"", "\\", " ", "\t"] {
                combinations.push(format!("{}{}{}", a, b, c));
            }
        }
    }
    let args: Vec<_> = fixtures
        .iter()
        .copied()
        .chain(combinations.iter().map(String::as_str))
        .collect();
    let expected = encode(args.iter().map(OsString::from));
    for quoted in [false, true] {
        let exe = std::env::current_exe().unwrap();
        let mut command = if quoted {
            argmax::Command::new_quoted(exe)
        } else {
            argmax::Command::new(exe)
        };
        command
            .try_arg("--echo")
            .unwrap()
            .try_args(args.as_slice())
            .unwrap();
        let output = command.output().unwrap();
        assert!(output.status.success());
        assert_eq!(output.stdout, expected);
        assert_eq!(
            command.get_args().skip(1).collect::<Vec<_>>(),
            args.iter().map(std::ffi::OsStr::new).collect::<Vec<_>>()
        );
    }

    let mut full = argmax::Command::new_quoted(std::env::current_exe().unwrap());
    full.try_arg("--echo").unwrap();
    let mut count = 0;
    while full.try_arg("\\\"'").is_ok() {
        count += 1;
    }
    let output = full.output().unwrap();
    assert!(output.status.success());
    assert_eq!(
        output.stdout,
        encode((0..count).map(|_| OsString::from("\\\"'")))
    );

    let printf = std::env::var_os("ARGMAX_MSYS_PRINTF")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| {
            std::path::PathBuf::from(
                std::env::var_os("ProgramFiles").unwrap_or_else(|| "C:/Program Files".into()),
            )
            .join("Git/usr/bin/printf.exe")
        });
    if printf.is_file() {
        let mut command = argmax::Command::new_quoted(printf);
        command
            .try_arg("%s\n")
            .unwrap()
            .try_args(args.as_slice())
            .unwrap();
        let output = command.output().unwrap();
        assert!(output.status.success(), "{:?}", output);
        assert_eq!(
            String::from_utf8(output.stdout).unwrap(),
            args.iter()
                .map(|arg| format!("{}\n", arg))
                .collect::<String>()
        );
    } else {
        eprintln!("MSYS2 test skipped: set ARGMAX_MSYS_PRINTF to an MSYS2 printf.exe");
    }
    println!("Windows argument round-trip checks passed");
}

#[cfg(not(windows))]
fn main() {}
