#![feature(asm)]
// main.rs - Entry point for FrameOS
#![no_std]
#![no_main]

extern crate alloc;
extern crate rlibc;

use alloc::{boxed::Box, rc::Rc, vec, vec::Vec};
use alloc::string::ToString;

use bootloader::{BootInfo, entry_point};
use x86_64::VirtAddr;

use frame_kernel::{
    clear_vga, print, println, serial_println,
    task::{executor::Executor, Task},
};
use frame_kernel::kcommand::CommandExecutor;
use frame_kernel::logger::Logger;
use frame_kernel::task::keyboard;
use frame_kernel::write_channel::stdout;

// define the entry point as kmain() instead of _start()
entry_point!(kmain);

// access the version as defined in cargo.toml
const VERSION: &'static str = env!("CARGO_PKG_VERSION");

// ================= ENTRY POINT

fn kmain(boot_info: &'static BootInfo) -> ! {
    use frame_kernel::allocator;
    use frame_kernel::memory::{self, BootInfoFrameAllocator};

    println!(
        "&bFrame&3OS &5v&d{} &9By &3Eric (Sk3pz) &9&& &3Matthew (MooCow9M)\n",
        VERSION
    );
    frame_kernel::init(); // initialize the interrupt handlers

    // the physical memory offset
    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    // used for translating virtual addresses (mapper.translate_addr(virtual address))
    let mut mapper = unsafe { memory::init(phys_mem_offset) };

    // create the frame allocator
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map) };
    // initialize the heap
    allocator::init_heap(&mut mapper, &mut frame_allocator)
        .expect("FrameOS Heap initialization failed.");

    // ================= MAIN RUNTIME CODE


    let logger = Logger::new(&stdout);



    let mut executor = Executor::new();

    executor.spawn(Task::new(keyboard::print_keypresses())); // enables keyboard input

    executor.run();

    // ================= HLT CODE
    // // print a message to show that everything worked
    // println!("All processes terminated. hlt loop executing until power down.");
    // // execute a hlt loop until shutdown
    // frame_os::hlt_loop();
}
