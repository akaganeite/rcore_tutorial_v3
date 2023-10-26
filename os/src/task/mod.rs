//! Task management implementation
//!
//! Everything about task management, like starting and switching tasks is
//! implemented here.
//!
//! A single global instance of [`TaskManager`] called `TASK_MANAGER` controls
//! all the tasks in the whole operating system.
//!
//! A single global instance of [`Processor`] called `PROCESSOR` monitors running
//! task(s) for each core.
//!
//! A single global instance of [`PidAllocator`] called `PID_ALLOCATOR` allocates
//! pid for user apps.
//!
//! Be careful when you see `__switch` ASM function in `switch.S`. Control flow around this function
//! might not be what you expect.
mod context;
mod manager;
mod pid;
mod processor;
mod switch;
#[allow(clippy::module_inception)]
#[allow(rustdoc::private_intra_doc_links)]
mod task;

use crate::fs::{open_file, OpenFlags};
use crate::sbi::shutdown;
use alloc::sync::Arc;
pub use context::TaskContext;
use lazy_static::*;
pub use manager::{fetch_task, TaskManager};
use switch::__switch;
pub use task::TaskStatus;
use task::TaskControlBlock;
use crate::config::MAX_SYSCALL_NUM;
use crate::mm::{VirtPageNum,MapPermission,VirtAddr,VPNRange};
use crate::timer::get_time_us;

pub use manager::add_task;
pub use pid::{pid_alloc, KernelStack, PidAllocator, PidHandle};
pub use processor::{
    current_task, current_trap_cx, current_user_token, run_tasks, schedule, take_current_task,
    Processor,
};
/// Suspend the current 'Running' task and run the next task in task list.
pub fn suspend_current_and_run_next() {
    // There must be an application running.
    let task = take_current_task().unwrap();

    // ---- access current TCB exclusively
    let mut task_inner = task.inner_exclusive_access();
    let task_cx_ptr = &mut task_inner.task_cx as *mut TaskContext;
    // Change status to Ready
    task_inner.task_status = TaskStatus::Ready;
    //task_inner.my_time+=get_time_us()-task_inner.timer;
    drop(task_inner);
    // ---- release current PCB

    // push back to ready queue.
    add_task(task);
    // jump to scheduling cycle
    schedule(task_cx_ptr);
}

/// pid of usertests app in make run TEST=1
pub const IDLE_PID: usize = 0;

/// Exit the current 'Running' task and run the next task in task list.
pub fn exit_current_and_run_next(exit_code: i32) {
    // take from Processor
    let task = take_current_task().unwrap();

    let pid = task.getpid();
    if pid == IDLE_PID {
        println!(
            "[kernel] Idle process exit with exit_code {} ...",
            exit_code
        );
        if exit_code != 0 {
            //crate::sbi::shutdown(255); //255 == -1 for err hint
            shutdown(true)
        } else {
            //crate::sbi::shutdown(0); //0 for success hint
            shutdown(false)
        }
    }

    // **** access current TCB exclusively
    let mut inner = task.inner_exclusive_access();
    // Change status to Zombie
    inner.task_status = TaskStatus::Zombie;
    // Record exit code
    inner.exit_code = exit_code;
    // do not move to its parent but under initproc

    //  access initproc TCB exclusively
    {
        let mut initproc_inner = INITPROC.inner_exclusive_access();
        for child in inner.children.iter() {
            child.inner_exclusive_access().parent = Some(Arc::downgrade(&INITPROC));
            initproc_inner.children.push(child.clone());
        }
    }
    //  release parent PCB

    inner.children.clear();
    // deallocate user space
    inner.memory_set.recycle_data_pages();
    inner.my_time+=get_time_us()-inner.timer;
    inner.timer=get_time_us();
    drop(inner);
    // **** release current PCB
    // drop task manually to maintain rc correctly
    drop(task);
    // we do not have to save task context
    let mut _unused = TaskContext::zero_init();
    schedule(&mut _unused as *mut _);
}

lazy_static! {
    ///Globle process that init user shell
    pub static ref INITPROC: Arc<TaskControlBlock> = Arc::new({
        let inode = open_file("ch6b_initproc", OpenFlags::RDONLY).unwrap();
        let v = inode.read_all();
        TaskControlBlock::new(v.as_slice())
    });
}
///Add init process to the manager
pub fn add_initproc() {
    add_task(INITPROC.clone());
}

///返回task_info
pub fn get_task_info()->([u32; MAX_SYSCALL_NUM],usize,TaskStatus) {
    let cur = current_task().unwrap();
    let mut inner=cur.inner_exclusive_access(); 
    inner.my_time+=get_time_us()-inner.timer;
    inner.timer=get_time_us();
    (inner.syscall_times,inner.my_time,inner.task_status)
    }
    
///增加syscall次数
pub fn add_syscall_num(id:usize){
    let cur = current_task().unwrap();
    let mut inner=cur.inner_exclusive_access();
    inner.syscall_times[id]+=1;
}


///check_whether_a_VPN_is_valid
pub fn check_page_validity(vpn:VirtPageNum)->usize{
    let cur = current_task().unwrap();
    let  inner=cur.inner_exclusive_access();    
    let mut flag=1;
    if let Some(pte)=inner.memory_set.translate(vpn){
        if pte.is_valid() {
            flag=0;
        }
    }
    //println!("flag:{}",flag);
    flag
}



///insert_vpnarea_to_memoryset
pub fn insert_framed_area(start_va: VirtAddr,end_va: VirtAddr,permission: MapPermission){
    let cur = current_task().unwrap();
    let mut inner=cur.inner_exclusive_access();    
    inner.memory_set.insert_framed_area(start_va, end_va, permission);
}


#[allow(unused)]
///unmap_a_range
pub fn unmap_range(range:VPNRange){
    let cur = current_task().unwrap();
    let mut inner=cur.inner_exclusive_access();
    for vpn in range{
        inner.memory_set.get_page_table().unmap(vpn);
    }
}