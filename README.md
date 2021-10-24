## argmax

`argmax` is a library that allows Rust applications to avoid *Argument list too long* errors (`E2BIG`) by providing a `std::process::Command` wrapper with a
``` rust
fn try_arg<S: AsRef<OsStr>>(&mut self, arg: S) -> bool
```
function that returns `false` if `arg` would overflow the maximum size.

## Resources

- http://mywiki.wooledge.org/BashFAQ/095
- https://www.in-ulm.de/~mascheck/various/argmax/
- https://github.com/tavianator/bfs/blob/9b50adaaaa4fedc8bda6fcf32595ecf7a682fa8b/exec.c#L72
- https://stackoverflow.com/questions/46897008/why-am-i-getting-e2big-from-exec-when-im-accounting-for-the-arguments-and-the
- https://github.com/rust-lang/rust/issues/40384
- `xargs --show-limits`
