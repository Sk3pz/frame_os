#![no_std]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]
#![feature(const_fn)]
#![feature(const_in_array_repeat_expressions)]
#![feature(wake_trait)]
#![feature(asm)]

extern crate alloc;

use core::panic::PanicInfo;

pub mod allocator;
pub mod gdt;
pub mod interrupts;
pub mod memory;
pub mod serial; // For use in debugging and testing ONLY! Not for use in main OS threads.
pub mod task;
pub mod vga_buffer;

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

// ================= CUSTOM PANIC IMPLIMENTATION

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    println!("&4{}", _info);

    hlt_loop();
}

// ================= HLT LOOP

pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}