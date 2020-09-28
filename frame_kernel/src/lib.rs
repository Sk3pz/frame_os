#![no_std]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]
#![feature(const_fn)]
#![feature(const_in_array_repeat_expressions)]
#![feature(wake_trait)]
#![feature(asm)]
#![feature(c_variadic)]

extern crate alloc;

use core::panic::PanicInfo;

use x86_64::instructions::port::Port;

pub mod allocator;
pub mod gdt;
pub mod interrupts;
pub mod memory;
pub mod serial; // For use in debugging and testing ONLY! Not for use in main OS threads.
pub mod task;
pub mod system;
pub mod logger;
pub mod write_channel;
pub mod vga_textmode;
pub mod syscalls;
pub mod logo_print;

// ================= HEAP ALLOCATION

pub const HEAP_START: usize = 0x_4444_4444_0000; // TODO: Handle this by not just setting it to a 'random' location
pub const HEAP_SIZE: usize = 500 * 1024; // 500 KiB

// ================= INITIALIZATION

pub fn init() {
    gdt::init();
    interrupts::init_idt();
    unsafe { interrupts::PICS.lock().initialize() };
    x86_64::instructions::interrupts::enable();
}

// ================= ALLOCATION ERROR HANDLING

#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("allocation error: {:?}", layout)
}

// ================= QEMU EXIT HANDLING (FOR DEBUGGING)

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

/**
    Will Exit the QEMU VM
    NOT FOR USE IN MAIN OS THREADS - TESTS AND DEBUGGING ONLY
**/
// For use in debugging and testing ONLY! Not for use in main OS threads.
pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;

    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
}

// ================= Port IO

/// Unsafe due to call to Port.write(...)
/// Unsafe due to writing to port
pub unsafe fn outb(port: u16, data: u16) {
    let mut p = Port::new(port);
    p.write(data);
}

/// Unsafe due to call to Port.read(...)
/// Unsafe due to writing to port
pub unsafe fn inb(port: u16) -> u16 {
    let mut p = Port::new(port);
    p.read()
}

// ================= CUSTOM PANIC IMPLIMENTATION

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! { // TODO: Timestamps
    println!("&4{}", _info);

    hlt_loop(); // halt the os
}

// ================= HLT LOOP

pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}
