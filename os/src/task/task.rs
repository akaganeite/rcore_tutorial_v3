//! Types related to task management

use super::TaskContext;
use crate::config::MAX_SYSCALL_NUM;
#[derive(Copy, Clone)]
pub struct TaskControlBlock {
    pub task_status: TaskStatus,
    pub task_cx: TaskContext,
    ///syscall_times
    pub syscall_times: [u32; MAX_SYSCALL_NUM],
    ///my_time
    pub my_time:usize,
    ///user_time_off
    pub user_time_off:usize,
    ///kern_time_off
    pub kern_time_off:usize,
}

#[derive(Copy, Clone, PartialEq)]
/// task status: UnInit, Ready, Running, Exited
pub enum TaskStatus {
    ///未初始化
    UnInit,
    ///就绪
    Ready,
    ///运行
    Running,
    ///退出
    Exited,
}
