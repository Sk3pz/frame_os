# frame_os
FrameOS is an operating system using the Rust programming language with speed and security in mind. This project's design goal is to bring a new, lightweight OS that doesn't get in the way of your daily tasks.

Building and Running:

Current Versions:

1.) Ensure QEMU is installed:
    - download from https://www.qemu.org/ and install
    - ensure qemu is added to path

2.) Installing BootImage:
    
    - run the command `rustup component add llvm-tools-preview`
    
    - install bootimage with `cargo install bootimage` 
      (must be run outside of the project directory due to our forced target)

3.) Set the rust toolchain version to nightly to gain access to important features
    
    - run the command `rustup override add nightly` or `rustup override set nightly` 
      if already installed.
        (can be set back to stable with `rustup override set stable`)

4.) install the rust source so the kernel can recompile parts it uses with 
    `rustup component add rust-src` 
    (must be run inside the project directory)

5a.) Inside of `\frame_kernel`, you can use `cargo run` or `cargo build`

6b.) To run the kernel, use `build.bat` or `run.bat`

```
*** FUTURE PLANNED BUILDING AND RUNNING ***
(This is how we plan to handle building and running after work on our own custom bootloader is finished)
** Build scripts are for Windows Only, however they can easily be modified for linux use separatly **
1.) Ensure Rust, Cargo, and QEMU are installed properly (and added to path if on Windows)
2.) Run `build.bat` or `run.bat` 
    (the run script executes the build script before running)
```

----------------
Known bugs:

 - VGA Text mode input (in kernel) - Backspacing a newline breaks input (requires restart)
 
In Process:

 - Simple File System
 - Custom Bootloader

TODO:
 - Background color setting through print statements (VGA-Text Mode in the kernel)
 - VGA Driver / Display Driver
 - command processing
 - built-in assembler
 - built-in c compiler
 - application running (make / run applications on the os, currently all processes are just the kernel)
 - more drivers
 - ext4 and NTFS file systems
 - better memory management
 - Support for .exe file execution
 - ABI
 - Operating System:
   * Drivers
   * Scheduler
   * etc
