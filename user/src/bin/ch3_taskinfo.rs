#![no_std]
#![no_main]

extern crate user_lib;

use user_lib::{
    get_time, println, sleep, task_info, TaskInfo, TaskStatus, SYSCALL_EXIT, SYSCALL_GETTIMEOFDAY,
    SYSCALL_TASK_INFO, SYSCALL_WRITE, SYSCALL_YIELD,
};

#[no_mangle]
pub fn main() -> usize {
    let t1 = get_time() as usize;
    let info = TaskInfo::new();
    get_time();
    sleep(500);
    let t2 = get_time() as usize;
    // 注意本次 task info 调用也计入
    assert_eq!(0, task_info(&info));
    let t3 = get_time() as usize;
    //println!("userinfo:{},offinfo:{},t2:{}",info.time,info.off_time,t2);
    assert!(3 <= info.syscall_times[SYSCALL_GETTIMEOFDAY]);
    assert_eq!(1, info.syscall_times[SYSCALL_TASK_INFO]);
    assert_eq!(0, info.syscall_times[SYSCALL_WRITE]);
    assert!(0 < info.syscall_times[SYSCALL_YIELD]);
    assert_eq!(0, info.syscall_times[SYSCALL_EXIT]);
    assert!(t2 - t1 <= info.time + 1);
    assert!(info.time < t3 - t1 + 100);
    assert!(info.status == TaskStatus::Running);

    // 想想为什么 write 调用是两次
    println!("string from task info test\n");
    let t4 = get_time() as usize;
    assert_eq!(0, task_info(&info));
    let t5 = get_time() as usize;
    assert!(5 <= info.syscall_times[SYSCALL_GETTIMEOFDAY]);
    assert_eq!(2, info.syscall_times[SYSCALL_TASK_INFO]);
    assert_eq!(2, info.syscall_times[SYSCALL_WRITE]);
    assert!(0 < info.syscall_times[SYSCALL_YIELD]);
    assert_eq!(0, info.syscall_times[SYSCALL_EXIT]);
    println!("userinfo:{},offinfo:{}",info.time,info.off_time);
    //assert!(t4 - t1 <= info.time + 1);
    assert!(info.time < t5 - t1 + 100);
    assert!(info.status == TaskStatus::Running);

    println!("Test task info OK4379742546335833259219086594142866576451913113532593393490644900559613255912425223451023416466595945032357688353922464875794578245330633032617045442586349091131960298342171549646298606125642406081067620695314801380557736324928055352076155360351650254569638057521226122940893442678345100476311936392928454028203953664649526221328472449298926628073717236167158792844639312214495569050399530853663942008!");
    0
}
