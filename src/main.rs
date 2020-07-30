// main.rs - Entry point for FrameOS
#![no_std]
#![no_main]

extern crate rlibc;
extern crate alloc;

use frame_os::{println, print, task::{Task, executor::Executor, keyboard}};
use x86_64::VirtAddr;
use bootloader::{BootInfo, entry_point};
use alloc::{boxed::Box, vec, vec::Vec, rc::Rc};

// ================= FEATURE TEST FUNCTIONS

async fn async_test_number() -> u32 {
    42
}

async fn async_test() {
    let number = async_test_number().await;
    println!("async number: {}", number);
}

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
    println!("&6current reference count is &e{}", Rc::strong_count(&cloned_reference));
    core::mem::drop(reference_counted);
    println!("&6reference count is &e{} &6now", Rc::strong_count(&cloned_reference));
}

// define the entry point as kmain() instead of _start()
entry_point!(kmain);

// access the version as defined in cargo.toml
const VERSION: &'static str = env!("CARGO_PKG_VERSION");

async fn loop_thing() {
    loop {
        print!("."); // TODO: Implement wait or sleep functions
    }
}

// ================= ENTRY POINT

fn kmain(boot_info: &'static BootInfo) -> ! {
    use frame_os::allocator;
    use frame_os::memory::{self, BootInfoFrameAllocator};

    // ================= KERNEL INITIALIZATION
    // print version info to the screen
    // minimal version:
    //println!("{}\n&7| &bFrame&3OS &7| &5Version &d{} &7| &2Author: &aEric Shreve&e &7|\n{}",
    //"&7+--------------------------------------------------+", VERSION,
    //"&7+--------------------------------------------------+");
    // even more minimal:
    println!("&bFrame&3OS &5v&d{} &2By &aEric Shreve&7", VERSION);
    // full version:
    //println!("{}\n&7> &bFrame&3OS\n&7> &5Version &d{} \n&7> &2Author: &aEric Shreve&e\n{}",
    //"&7----------------------", VERSION,
    //"&7----------------------");
    frame_os::init(); // initialize the interrupt handlers

    // the physical memory offset
    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    // used for translating virtual addresses (mapper.translate_addr(virtual address))
    let mut mapper = unsafe {memory::init(phys_mem_offset)};
    // create the frame allocator
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map) };
    // initialize the heap
    allocator::init_heap(&mut mapper, &mut frame_allocator)
        .expect("FrameOS Heap initialization failed.");


    // ================= MAIN RUNTIME CODE 

    let mut executor = Executor::new();
    // executor.spawn(Task::new(async_test()));
    // executor.spawn(Task::new(heap_test()));
    //executor.spawn(Task::new(loop_thing()));
    executor.spawn(Task::new(keyboard::print_keypresses()));

    executor.run();

    // ================= HLT CODE
    // // print a message to show that everything worked
    // println!("All processes terminated. hlt loop executing until power down.");
    // // execute a hlt loop until shutdown
    // frame_os::hlt_loop();
}
