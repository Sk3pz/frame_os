// main.rs - Entry point for FrameOS
#![no_std]
#![no_main]

extern crate alloc;
extern crate rlibc;

use alloc::{boxed::Box, rc::Rc, vec, vec::Vec};

use bootloader::{BootInfo, entry_point};
use x86_64::VirtAddr;

use frame_kernel::{
    print, println,
    task::{executor::Executor, Task},
};
use frame_kernel::logger::Logger;
use frame_kernel::write_channel::ChannelSTDERR;
use frame_kernel::write_channel::ChannelSTDIN;
use frame_kernel::write_channel::ChannelSTDOUT;

// ================= FEATURE TEST FUNCTIONS

async fn heap_test() {
    // allocate a number on the heap
    let heap_value = Box::new(42);
    println!("&6heap_value at &e{:p}", heap_value);

    // create a dynamically sized vector
    let mut vec = Vec::new();
    for i in 0..500 {
        vec.push(i);
    }
    println!("&6vec at &e{:p}", vec.as_slice());
    vec.push(501);
    println!("&6vec at &e{:p}", vec.as_slice());

    // create a reference counted vector -> will be freed when count reaches 0
    let reference_counted = Rc::new(vec![1, 2, 3]);
    let cloned_reference = reference_counted.clone();
    println!(
        "&6current reference count is &e{}",
        Rc::strong_count(&cloned_reference)
    );
    core::mem::drop(reference_counted);
    println!(
        "&6reference count is &e{} &6now",
        Rc::strong_count(&cloned_reference)
    );
}

// define the entry point as kmain() instead of _start()
entry_point!(kmain);

// access the version as defined in cargo.toml
const VERSION: &'static str = env!("CARGO_PKG_VERSION");

// ================= ENTRY POINT

fn kmain(boot_info: &'static BootInfo) -> ! {
    use frame_kernel::allocator;
    use frame_kernel::memory::{self, BootInfoFrameAllocator};

    println!(
        "&bFrame&3OS &5v&d{} &9By &3Eric (Sk3pz) &9&& &3Matthew (MooCow9M)&7",
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

    // Initialize output channels
    let stdout = ChannelSTDOUT {};
    let stdin = ChannelSTDIN {};
    let stderr = ChannelSTDERR {};

    // ================= MAIN RUNTIME CODE

    let logger = Logger::new(&stdout);

    logger.debug("This is a debug logging test");
    logger.verbose("This is a verbose logging test");
    logger.info("This is an info logging test");
    logger.warn("This is a warning logging test");
    logger.error("This is an error logging test");
    logger.wtf("This is a failure logging test");


    let mut executor = Executor::new();
    // executor.spawn(Task::new(heap_test()));
    // executor.spawn(Task::new(keyboard::print_keypresses())); // enables keyboard input

    executor.run();

    // ================= HLT CODE
    // // print a message to show that everything worked
    // println!("All processes terminated. hlt loop executing until power down.");
    // // execute a hlt loop until shutdown
    // frame_os::hlt_loop();
}
