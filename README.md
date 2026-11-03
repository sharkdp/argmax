# argmax

<a href="https://crates.io/crates/argmax"><img src="https://img.shields.io/crates/v/argmax.svg?colorB=319e8c" alt="Version info"></a> <a href="https://docs.rs/argmax"><img src="https://docs.rs/argmax/badge.svg"></a> [![CICD](https://github.com/sharkdp/argmax/actions/workflows/CICD.yml/badge.svg)](https://github.com/sharkdp/argmax/actions/workflows/CICD.yml)

`argmax` is a library that allows Rust applications to avoid *Argument list too long* errors (`E2BIG`) by providing a `std::process::Command` wrapper with a
``` rust
fn try_arg<S: AsRef<OsStr>>(&mut self, arg: S) -> io::Result<&mut Self>
```
function that returns a proper `Err`or if `arg` would overflow the maximum size.

## Windows arguments

`Command::new_quoted` is an opt-in constructor for passing literal arguments to
MSYS2/Cygwin programs. It also preserves arguments for native programs using the
Windows C runtime. Quoting and escaping are included in the argument-size budget.
Use the ordinary `Command::new` for `cmd.exe` and batch files, which have different
escaping rules.

The Windows argument tests use Git for Windows's `printf.exe` when available.
Set `ARGMAX_MSYS_PRINTF` to test another MSYS2 installation. Native Windows
argument checks run independently of whether MSYS2 is installed.

## Resources

This library draws inspiration from the following sources. The implementation is based on
the corresponding functionality in [`bfs`](https://github.com/tavianator/bfs) [1].

- [1] https://github.com/tavianator/bfs/blob/9b50adaaaa4fedc8bda6fcf32595ecf7a682fa8b/exec.c#L72
- [2] http://mywiki.wooledge.org/BashFAQ/095
- [3] https://www.in-ulm.de/~mascheck/various/argmax/
- [4] https://stackoverflow.com/questions/46897008/why-am-i-getting-e2big-from-exec-when-im-accounting-for-the-arguments-and-the
- [5] https://github.com/rust-lang/rust/issues/40384
- [6] `xargs --show-limits`

## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
