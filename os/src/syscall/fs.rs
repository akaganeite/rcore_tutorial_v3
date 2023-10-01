//! File and filesystem-related syscalls
const APP_BASE_ADDRESS: usize = 0x80400000;
const APP_SIZE_LIMIT: usize = 0x20000;
const USER_STACK_SIZE: usize = 4096;
const FD_STDOUT: usize = 1;
use crate::batch::USER_STACK;
/// write buf of length `len`  to a file with `fd`
pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    match fd {
        FD_STDOUT => {
            let usp=USER_STACK.get_sp();
            let ubuf=buf as usize;
            if ubuf+len<usp&&ubuf>=(usp-USER_STACK_SIZE) {
                
            }
            else if ubuf>=APP_BASE_ADDRESS&&ubuf+len<=APP_BASE_ADDRESS+APP_SIZE_LIMIT {
                
            }
            else {
                println!("accessing illegal address");
                return -1;
            }
            //println!("buf:{:#x}\nsp:{:#x}",buf as usize,USER_STACK.get_sp());
            let slice = unsafe { core::slice::from_raw_parts(buf, len) };
            let str = core::str::from_utf8(slice).unwrap();
            print!("{}", str);
            len as isize
        }
        _ => {
            //panic!("Unsupported fd in sys_write!");
            -1
        }
    }
}
