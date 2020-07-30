# frame_os
An x86_64 operating system written in rust

To build:

1.) install some dependencies with `rustup component add llvm-tools-preview`

2.) install bootimage with `cargo install bootimage`

3.) Use the latest Rust Nightly target, and run `cargo build`. `cargo run` will also work.

Dont have rust nightly? 
install with `rustup override add nightly`
