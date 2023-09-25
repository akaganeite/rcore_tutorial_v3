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

    }
    logger::init_logger();
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