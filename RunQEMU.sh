#!/bin/sh
echo "Please ensure VcXsrv is running if on windows, with an opened window with the display ID of 0:0"
echo "Setting display to VcXsrv Server Display 0:0"
export DISPLAY=0:0
echo "Display has been set for QEMU"
echo "Running QEMU x86_64 VM for FrameOS. Press \`CTRL+C\` to exit."
qemu-system-x86_64 -drive format=raw,file=target/x86_64-frame_os/debug/bootimage-frame_os.bin