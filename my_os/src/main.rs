#![no_std]
#![no_main]
#![feature(panic_info_message)]
#[macro_use]
mod console;
mod lang_items;
mod logger;
mod sbi;
use core::arch::global_asm;
use log::*;
//use sbi::{shutdown, console_putchar};
global_asm!(include_str!("entry.asm"));

#[no_mangle]
pub fn rust_main()->! {
    clear_bss();
    extern "C" {
        fn stext();
        fn etext();
        fn srodata();
        fn erodata();
        fn sdata();
        fn edata();
        fn sbss();
        fn ebss();
        fn boot_stack_top();
        fn boot_stack_lower_bound();
    }
    logger::init_logger();
    info!("[kernel].text [{:#x}, {:#x})", stext as usize, etext as usize);
    info!("[kernel].rodata [{:#x}, {:#x})", srodata as usize, erodata as usize);
    info!("[kernel].data [{:#x}, {:#x})", sdata as usize, edata as usize);
    info!("[kernel].bss [{:#x}, {:#x})", sbss as usize, ebss as usize);
    info!("[kernel].boot_stack lower_bound:{:#x}, top:{:#x}", boot_stack_lower_bound as usize, boot_stack_top as usize);
    info!("Hello_world");
    panic!("shutdown");
}

fn clear_bss(){
    extern "C" {
        fn sbss();
        fn ebss();
    }
    (sbss as usize..ebss as usize).for_each(|a|{//对整片区域逐字节置零，1u8就是1个字节
        unsafe{(a as *mut u8).write_volatile(0)}
    });
}