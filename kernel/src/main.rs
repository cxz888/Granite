#![no_std]
#![no_main]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]
#![feature(step_trait)]
#![feature(assert_matches)]
#![feature(let_chains)]

extern crate alloc;

#[macro_use]
mod utils;
mod config;
mod driver_impl;
mod fs;
mod loader;
mod memory;
mod signal;
mod sync;
mod syscall;
mod task;
mod trap;

core::arch::global_asm!(include_str!("entry.asm"));

/// clear BSS segment
fn clear_bss() {
    extern "C" {
        fn sbss();
        fn ebss();
    }
    unsafe {
        core::slice::from_raw_parts_mut(sbss as usize as *mut u8, ebss as usize - sbss as usize)
            .fill(0);
    }
}

#[no_mangle]
/// the rust entry-point of os
pub fn rust_main() -> ! {
    clear_bss();
    utils::logging::init();
    println!("[kernel] Hello, world!");
    memory::init();
    trap::init();
    trap::enable_timer_interrupt();
    utils::time::set_next_trigger();
    fs::list_apps();
    task::add_initproc();
    task::run_tasks();
    panic!("Unreachable in rust_main!");
}
