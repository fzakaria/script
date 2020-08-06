# sciptr

This is simple re-implementation of the [script](https://man7.org/linux/man-pages/man1/script.1.html) tool written
in Rust.

*"script makes a typescript of everything on your terminal session."*

> This is one of my first Rust programs so please be gentle when evaluating the codebase.


## Running the program

You can run _scriptr_ **easily** via Nix.

```bash
nix run -f https://github.com/fzakaria/scriptr/archive/master.tar.gz --command scriptr
```

If you have the Rust toolchain ready to go, you can also just run
```bash
cargo run
```