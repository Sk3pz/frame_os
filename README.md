# frame_os
An x86_64 operating system written in rust

To build:

1.) install some dependencies for bootimage and this OS with `rustup component add llvm-tools-preview`

2.) install bootimage with `cargo install bootimage`

3.) Use the latest Rust Nightly target, and run `cargo build`. `cargo run` will also work.

Dont have rust nightly? Install with `rustup override add nightly`

(can be reset with `rustup override set stable`)


----------------
Known bugs:

 - Backspacing a newline breaks input (requires restart)
 
 
TODO:
 - Background color setting through print statements
 - VGA Driver
 - Custom Bootloader (using bootimage crate for rust)
 - command processing
 - gcc support
 - application running (make / run applications on the os, currently all processes are just the kernel)
 - more drivers
 - file system
 - better memeory management
