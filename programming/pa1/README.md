Will Usher - Programming Assignment 1
-

## Compilation

To compile the program you will need the Rust compiler and its Cargo toolchain
(included with Rust). You can find a download for the most recent version of Rust
at [rust-lang.org](https://www.rust-lang.org/en-US/downloads.html) along with
more information about the language.

Once Rust is installed both the `rustc` and `cargo` commands should be in your path.
To build the project in release mode cd into the project directory (with the `Cargo.toml` file)
and run `cargo build --release`. This will take a few minutes as some dependencies are downloaded
and compiled locally. After compiling the project you can run it with `cargo run --release`
or directly find the binary under `./target/release/bezier`.

## Controls

