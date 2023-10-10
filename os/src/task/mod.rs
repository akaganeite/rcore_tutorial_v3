//! Task management implementation
//!
//! Everything about task management, like starting and switching tasks is
//! implemented here.
//!
//! A single global instance of [`TaskManager`] called `TASK_MANAGER` controls
//! all the tasks in the operating system.
//!
//! Be careful when you see `__switch` ASM function in `switch.S`. Control flow around this function
//! might not be what you expect.

mod context;
mod switch;

#[allow(clippy::module_inception)]
mod task;

use crate::config::{MAX_APP_NUM,MAX_SYSCALL_NUM};
use crate::loader::{get_num_app, init_app_cx};
use crate::sbi::shutdown;
use crate::sync::UPSafeCell;
use lazy_static::*;
use switch::__switch;
use crate::timer::get_time_ms;
use task::TaskControlBlock;
pub use task::TaskStatus;
pub use context::TaskContext;

/// The task manager, where all the tasks are managed.
///
/// Functions implemented on `TaskManager` deals with all task state transitions
/// and task context switching. For convenience, you can find wrappers around it
/// in the module level.
///
/// Most of `TaskManager` are hidden behind the field `inner`, to defer
/// borrowing checks to runtime. You can see examples on how to use `inner` in
/// existing functions on `TaskManager`.
pub struct TaskManager {
    /// total number of tasks
    num_app: usize,
    /// use inner value to get mutable access
    inner: UPSafeCell<TaskManagerInner>,
}

/// Inner of Task Manager
pub struct TaskManagerInner {
    /// task list
    tasks: [TaskControlBlock; MAX_APP_NUM],
    /// id of current `Running` task
    current_task: usize,
    ///计时器
    timer:usize,
    ///my_timer
    my_timer:usize,
}

lazy_static! {
    /// Global variable: TASK_MANAGER
    pub static ref TASK_MANAGER: TaskManager = {
        let num_app = get_num_app();
        let mut tasks = [TaskControlBlock {//列表中所有的控制块都初始化为0和uninit
            task_cx: TaskContext::zero_init(),//初始化为全0
            task_status: TaskStatus::UnInit,
            syscall_times: [0;MAX_SYSCALL_NUM],
            my_time:0,
            user_time_off:0,
            kern_time_off:0,
        }; MAX_APP_NUM];
        for (i, task) in tasks.iter_mut().enumerate() {
            task.task_cx = TaskContext::goto_restore(init_app_cx(i));
            //init把trapcontext压入内核栈，goto设置taskcontext的ra和sp
            task.task_status = TaskStatus::Ready;//装载后是就绪态
        }
        TaskManager {
            num_app,
            inner: unsafe {
                UPSafeCell::new(TaskManagerInner {
                    timer:0,
                    my_timer:0,
                    tasks,
                    current_task: 0,//从0号task开始执行
                })
            },
        }
    };
}

impl TaskManager {
    /// Run the first task in task list.
    ///
    /// Generally, the first task in task list is an idle task (we call it zero process later).
    /// But in ch3, we load apps statically, so the first task is a real app.
    fn run_first_task(&self) -> ! {
        let mut inner = self.inner.exclusive_access();
        let task0 = &mut inner.tasks[0];
        task0.task_status = TaskStatus::Running;
        let next_task_cx_ptr = &task0.task_cx as *const TaskContext;

        //初始化timer，进入用户态，等待下次返回
        //inner.refresh_my_watch();
        inner.refresh_stop_watch();
        inner.my_timer=get_time_ms();

        drop(inner);
        let mut _unused = TaskContext::zero_init();
        // before this, we should drop local variables that must be dropped manually
        unsafe {//当前部分状态保存到unused指向的地址
            __switch(&mut _unused as *mut TaskContext, next_task_cx_ptr);
        }//switch结束后ret到restore
        panic!("unreachable in run_first_task!");
    }

    /// Change the status of current `Running` task into `Ready`.
    fn mark_current_suspended(&self) {
        let mut inner = self.inner.exclusive_access();
        let current = inner.current_task;
        inner.tasks[current].kern_time_off += inner.refresh_stop_watch();
        //inner.tasks[current].my_time += inner.refresh_my_watch();
        inner.tasks[current].task_status = TaskStatus::Ready;
    }

    /// Change the status of current `Running` task into `Exited`.
    fn mark_current_exited(&self) {
        let mut inner = self.inner.exclusive_access();
        let current = inner.current_task;
        inner.tasks[current].kern_time_off += inner.refresh_stop_watch();
        inner.tasks[current].task_status = TaskStatus::Exited;
    }

    /// Find next task to run and return task id.
    ///
    /// In this case, we only return the first `Ready` task in task list.
    fn find_next_task(&self) -> Option<usize> {
        let inner = self.inner.exclusive_access();
        let current = inner.current_task;
        (current + 1..current + self.num_app + 1)
            .map(|id| id % self.num_app)//从当前id开始，找到当前id-1 
            .find(|id| inner.tasks[*id].task_status == TaskStatus::Ready)
    }

    /// Switch current `Running` task to the task we have found,
    /// or there is no `Ready` task and we can exit with all applications completed
    fn run_next_task(&self) {
        if let Some(next) = self.find_next_task() {
            let mut inner = self.inner.exclusive_access();
            let current = inner.current_task;
            inner.tasks[next].task_status = TaskStatus::Running;
            inner.current_task = next;
            let current_task_cx_ptr = &mut inner.tasks[current].task_cx as *mut TaskContext;
            let next_task_cx_ptr = &inner.tasks[next].task_cx as *const TaskContext;
            
            let now=get_time_ms();
            inner.tasks[current].my_time+=now-inner.my_timer;
            inner.my_timer=get_time_ms();

            drop(inner);
            // before this, we should drop local variables that must be dropped manually
            unsafe {
                __switch(current_task_cx_ptr, next_task_cx_ptr);
            }
            // go back to user mode
        } else {
            let mut inner = self.inner.exclusive_access();
            let current = inner.current_task;
            let now=get_time_ms();
            inner.tasks[current].my_time+=now-inner.my_timer;
            inner.my_timer=get_time_ms();
            drop(inner);
            println!("All applications completed!");
            shutdown();
        }
    }

    fn stop_user_time(&self){
        let mut inner = self.inner.exclusive_access();
        let current = inner.current_task;
        inner.tasks[current].user_time_off += inner.refresh_stop_watch();
    
    }
    
    ///start_user_timer?
    fn start_user_time(&self){
        let mut inner = self.inner.exclusive_access();
        let current = inner.current_task;
    
        inner.tasks[current].kern_time_off += inner.refresh_stop_watch();
    }    
}


impl TaskManagerInner{
///timer_manage
fn refresh_stop_watch(&mut self) -> usize {
    let start_time = self.timer;
    self.timer = get_time_ms();
    self.timer - start_time
}


}


/// run first task
pub fn run_first_task() {
    TASK_MANAGER.run_first_task();
}

/// rust next task
fn run_next_task() {
    TASK_MANAGER.run_next_task();
}

/// suspend current task
fn mark_current_suspended() {
    TASK_MANAGER.mark_current_suspended();
}

/// exit current task
fn mark_current_exited() {
    TASK_MANAGER.mark_current_exited();
}

/// suspend current task, then run next task
pub fn suspend_current_and_run_next() {
    mark_current_suspended();
    run_next_task();
}

/// exit current task,  then run next task
pub fn exit_current_and_run_next() {
    mark_current_exited();
    run_next_task();
}

///返回task_info
pub fn get_task_info()->TaskControlBlock {
    let inner = TASK_MANAGER.inner.exclusive_access();
    let current = inner.current_task;
    inner.tasks[current]
}

///增加syscall次数
pub fn add_syscall_num(id:usize){
    let mut inner = TASK_MANAGER.inner.exclusive_access();
    let current = inner.current_task;
    inner.tasks[current].syscall_times[id]+=1;
}

///user_time_stop
pub fn user_time_stop(){
    TASK_MANAGER.stop_user_time();
}

///user_time_start
pub fn user_time_start(){
    TASK_MANAGER.start_user_time();
}