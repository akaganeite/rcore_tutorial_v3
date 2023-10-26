use crate::fs::{open_file, OpenFlags};
use crate::mm::{translated_refmut, translated_str,VirtAddr,VPNRange,MapPermission,translated_byte_buffer};
use crate::task::{
    add_task, current_task, current_user_token, exit_current_and_run_next,
    suspend_current_and_run_next,TaskStatus,get_task_info,check_page_validity,insert_framed_area,unmap_range
};
use core::mem::size_of;
use alloc::sync::Arc;
use crate::timer::get_time_us;
//use super::SYSCALL_GET_TIME;
use crate::config::{MAX_SYSCALL_NUM,PAGE_SIZE};

pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct TaskInfo {
    pub status: TaskStatus,
    pub syscall_times: [u32; MAX_SYSCALL_NUM],
    pub time: usize,
}



pub fn sys_exit(exit_code: i32) -> ! {
    exit_current_and_run_next(exit_code);
    panic!("Unreachable in sys_exit!");
}

pub fn sys_yield() -> isize {
    suspend_current_and_run_next();
    0
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


pub fn sys_getpid() -> isize {
    current_task().unwrap().pid.0 as isize
}

pub fn sys_fork() -> isize {
    let current_task = current_task().unwrap();
    let new_task = current_task.fork();
    let new_pid = new_task.pid.0;
    // modify trap context of new_task, because it returns immediately after switching
    let trap_cx = new_task.inner_exclusive_access().get_trap_cx();
    // we do not have to move to next instruction since we have done it before
    // for child process, fork returns 0
    trap_cx.x[10] = 0;
    // add new task to scheduler
    add_task(new_task);
    new_pid as isize
    
}

pub fn sys_exec(path: *const u8) -> isize {
    let token = current_user_token();
    let path = translated_str(token, path);
    if let Some(app_inode) = open_file(path.as_str(), OpenFlags::RDONLY) {
        let all_data = app_inode.read_all();
        let task = current_task().unwrap();
        task.exec(all_data.as_slice());
        0
    } else {
        -1
    }
}

/// If there is not a child process whose pid is same as given, return -1.
/// Else if there is a child process but it is still running, return -2.
pub fn sys_waitpid(pid: isize, exit_code_ptr: *mut i32) -> isize {
    let task = current_task().unwrap();
    // find a child process

    // ---- access current TCB exclusively
    let mut inner = task.inner_exclusive_access();
    if !inner
        .children
        .iter()
        .any(|p| pid == -1 || pid as usize == p.getpid())
    {
        return -1;
        // ---- release current PCB
    }
    let pair = inner.children.iter().enumerate().find(|(_, p)| {
        // ++++ temporarily access child PCB lock exclusively
        p.inner_exclusive_access().is_zombie() && (pid == -1 || pid as usize == p.getpid())
        // ++++ release child PCB
    });
    if let Some((idx, _)) = pair {
        let child = inner.children.remove(idx);
        // confirm that child will be deallocated after removing from children list
        assert_eq!(Arc::strong_count(&child), 1);
        let found_pid = child.getpid();
        // ++++ temporarily access child TCB exclusively
        let exit_code = child.inner_exclusive_access().exit_code;
        // ++++ release child PCB
        *translated_refmut(inner.memory_set.token(), exit_code_ptr) = exit_code;
        found_pid as isize
    } else {
        -2
    }
    // ---- release current PCB lock automatically
}

/// change data segment size
pub fn sys_sbrk(size: i32) -> isize {
    if let Some(old_brk) = current_task().unwrap().change_program_brk(size) {
        old_brk as isize
    } else {
        -1
    }
}

///get_task_info
pub fn sys_task_info(ti: *mut TaskInfo) -> isize {
    let buffers = translated_byte_buffer(current_user_token(), ti as *const u8, size_of::<TaskInfo>());
    let  (syscall_times,my_time,status)=get_task_info();
    let ti=TaskInfo{
        syscall_times,
        time:((((my_time/1_000_000) & 0xffff) * 1000 + (my_time%1_000_000) / 1000) as usize),
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

pub fn sys_mmap(start: usize, len: usize, prot: usize) -> isize{
    if start%PAGE_SIZE!=0||
       prot & !0x7!=0||
       prot & 0x7 == 0
    {return  -1;}
    let s_vpn=VirtAddr::from(start).floor();
    let e_vpn=VirtAddr::from(start+len).ceil();
    let range=VPNRange::new(s_vpn, e_vpn);
    for vpn in range{
        if check_page_validity(vpn)==0{
            println!("[FAIL][SYSCALL]sys_mmap,found_invalid_page|vpn:{:?}",vpn);
            return -1;
        }
   }
    insert_framed_area(s_vpn.into(), e_vpn.into(), MapPermission::from_bits_truncate((prot << 1) as u8) | MapPermission::U);
    
    0
} 

pub fn sys_munmap(start: usize, len: usize) -> isize{
    if start%PAGE_SIZE!=0 {return  -1;}
    let s_vpn=VirtAddr::from(start).floor();
    let e_vpn=VirtAddr::from(start+len).ceil();
    let range=VPNRange::new(s_vpn, e_vpn);
    for vpn in range{
        if check_page_validity(vpn)!=0{
            println!("[FAIL][SYSCALL]sys_mmap,found_invalid_page|vpn:{}",vpn.0);
            return -1;
        }
    }
    unmap_range(range);
    0
    //unmap_consecutive_area(start, len)
}

pub fn sys_spawn(path: *const u8) -> isize {
    let token = current_user_token();
    let path = translated_str(token, path);
    if let Some(app_inode) = open_file(path.as_str(), OpenFlags::RDONLY) {
        let all_data = app_inode.read_all();
        let current_task = current_task().unwrap();
        let new_task = current_task.spawn(all_data.as_slice());
        let new_pid = new_task.pid.0;
        let trap_cx = new_task.inner_exclusive_access().get_trap_cx();
        // we do not have to move to next instruction since we have done it before
        // for child process, fork returns 0
        trap_cx.x[10] = 0;
        add_task(new_task);
        new_pid as isize
    } else {
        -1
    } 
}