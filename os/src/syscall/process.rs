//! Process management syscalls

//use riscv::addr::page;

//use crate::task::{change_program_brk, exit_current_and_run_next, suspend_current_and_run_next};
use crate::task::{change_program_brk, exit_current_and_run_next, suspend_current_and_run_next,get_task_info,TaskStatus};
use crate::timer::get_time_us;
//use super::SYSCALL_GET_TIME;
use crate::config::{MAX_SYSCALL_NUM, PAGE_SIZE};
use crate::mm::{translated_byte_buffer, VirtAddr,VPNRange, VirtPageNum};
use crate::task::{current_user_token,check_page_validity,insert_framed_area};
use core::mem::size_of;
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}
  
#[derive(Copy, Clone,Debug)]
#[repr(C)]
pub struct TaskInfo {
    pub off_time:usize,
    pub status: TaskStatus,
    pub syscall_times: [u32; MAX_SYSCALL_NUM],
    pub time: usize,
 }

/// task exits and submit an exit code
pub fn sys_exit(exit_code: i32) -> ! {
    println!("[kernel] Application exited with code {}", exit_code);
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    suspend_current_and_run_next();
    0
}

// /// get current time
// pub fn sys_get_time() -> isize {
//     get_time_ms() as isize
// }

/// change data segment size
pub fn sys_sbrk(size: i32) -> isize {
    if let Some(old_brk) = change_program_brk(size) {
        old_brk as isize
    } else {
        -1
    }
}

///get current time
pub fn sys_get_time(ts: *mut TimeVal, _tz: usize) -> isize {
    let us: usize = get_time_us();
    let buffers = translated_byte_buffer(current_user_token(), ts as *const u8, size_of::<TimeVal>());
    let time=TimeVal {
        sec: us / 1_000_000,
        usec: us % 1_000_000,
    };
    let ptr= &time as *const TimeVal as *const  u8;
    unsafe{
        let src=core::slice::from_raw_parts(ptr,size_of::<TimeVal>());
        //let mut last=0;
        for buf in buffers{
            // let chosen=buf.len().min(src.len()-last);
            // buf.copy_from_slice(&src[last..last+chosen]);
            // last=last+chosen;
            buf.copy_from_slice(src);
        }
    }
    0
}


/// YOUR JOB: Finish sys_task_info to pass testcases
pub fn sys_task_info(ti: *mut TaskInfo) -> isize {
    let buffers = translated_byte_buffer(current_user_token(), ti as *const u8, size_of::<TaskInfo>());
    let  (syscall_times,my_time,user_time_off,kern_time_off,status)=get_task_info();
    let ti=TaskInfo{
        syscall_times,
        time:((((my_time/1_000_000) & 0xffff) * 1000 + (my_time%1_000_000) / 1000) as usize),
        off_time:user_time_off+kern_time_off,
        status,
    };
    let ptr= &ti as *const TaskInfo as *const  u8;
    unsafe{
        let src=core::slice::from_raw_parts(ptr,size_of::<TaskInfo>());
       let mut last=0;
        for buf in buffers{
            let chosen=buf.len().min(src.len()-last);
            buf.copy_from_slice(&src[last..last+chosen]);
            last=last+chosen;
            //buf.copy_from_slice(src);
        }
    }
    0
}

#[allow(unused)]
pub fn sys_mmap(start: usize, len: usize, prot: usize) -> isize{
    if(start%PAGE_SIZE!=0||
       prot & !0x7!=0||
       prot & 0x7 == 0
    ){return  -1;}
    let s_vpn=VirtPageNum::from(start);
    let e_vpn=VirtAddr::from(start+len).ceil();
    let range=VPNRange::new(s_vpn, e_vpn);
    for vpn in range{
        if(check_page_validity(vpn)==0){
            println!("[FAIL][SYSCALL]sys_mmap,found_valid_page|vpn:{}",vpn.0);
            return -1;
        }
    }
    
    insert_framed_area(s_vpn.into(), e_vpn.into(), permission);
    
    0
} 

#[allow(unused)]
pub fn sys_munmap(start: usize, len: usize) -> isize{
    0
}