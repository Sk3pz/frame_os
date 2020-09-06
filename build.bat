@echo off
echo Building Kernel...
rustup override add nightly
rustup override set nightly
cd frame_kernel
cargo build
cd ..
echo Done Building