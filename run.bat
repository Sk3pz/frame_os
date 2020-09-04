@echo off
CALL build.bat
echo Running Kernel...
cd frame_kernel
cargo run
cd ..
echo Done