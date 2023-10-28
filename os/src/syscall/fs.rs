//! File and filesystem-related syscalls
use crate::fs::{open_file, OpenFlags,Stat,ROOT_INODE};
use crate::mm::{translated_byte_buffer, translated_str, UserBuffer};
use crate::task::{current_task, current_user_token};
use core::mem::size_of;

pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    let token = current_user_token();
    let task = current_task().unwrap();
    let inner = task.inner_exclusive_access();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if let Some(file) = &inner.fd_table[fd] {
        if !file.writable() {
            return -1;
        }
        let file = file.clone();
        // release current task TCB manually to avoid multi-borrow
        drop(inner);
        file.write(UserBuffer::new(translated_byte_buffer(token, buf, len))) as isize
    } else {
        -1
    }
}

pub fn sys_read(fd: usize, buf: *const u8, len: usize) -> isize {
    let token = current_user_token();
    let task = current_task().unwrap();
    let inner = task.inner_exclusive_access();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if let Some(file) = &inner.fd_table[fd] {
        let file = file.clone();
        if !file.readable() {
            return -1;
        }
        // release current task TCB manually to avoid multi-borrow
        drop(inner);
        file.read(UserBuffer::new(translated_byte_buffer(token, buf, len))) as isize
    } else {
        -1
    }
}

pub fn sys_open(path: *const u8, flags: u32) -> isize {
    let task = current_task().unwrap();
    let token = current_user_token();
    let path = translated_str(token, path);
    if let Some(inode) = open_file(path.as_str(), OpenFlags::from_bits(flags).unwrap()) {
        let mut inner = task.inner_exclusive_access();
        let fd = inner.alloc_fd();
        inner.fd_table[fd] = Some(inode);
        fd as isize
    } else {
        -1
    }
}

pub fn sys_close(fd: usize) -> isize {
    let task = current_task().unwrap();
    let mut inner = task.inner_exclusive_access();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if inner.fd_table[fd].is_none() {
        return -1;
    }
    inner.fd_table[fd].take();
    0
}


pub fn sys_fstat(fd: usize, st: *mut Stat)-> isize{
    let task = current_task().unwrap();
    let inner = task.inner_exclusive_access();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if inner.fd_table[fd].is_none() {
        return -1;
    }
    if let Some(filenode)=&inner.fd_table[fd]{
        let stat=filenode.info();
        //current_user_token();会死锁
        let buffers = translated_byte_buffer(inner.get_user_token(), st as *const u8, size_of::<Stat>());
        let ptr= &stat as *const Stat as *const u8 ;
        unsafe{
            let src=core::slice::from_raw_parts(ptr,size_of::<Stat>());
            let mut last=0;
            for buf in buffers{
                let chosen=buf.len().min(src.len()-last);
                buf.copy_from_slice(&src[last..last+chosen]);
                last=last+chosen;
                //buf.copy_from_slice(src);
            }
        }
        0
    }else{
        -1
    }
}

pub fn sys_linkat(old_name: *const u8, new_name: *const u8) -> isize {
    let token = current_user_token();
    let old_path = translated_str(token, old_name);
    let new_path = translated_str(token, new_name);
    if old_path==new_path {return -1} 
    if let Some(_)=ROOT_INODE.link(old_path.as_str(), new_path.as_str()){
        return 0
    }
    -1
}

#[allow(unused)]
pub fn sys_unlinkat(name: *const u8) -> isize {
    let token = current_user_token();
    let path = translated_str(token, name);
    if let Some(inode) = ROOT_INODE.find(path.as_str()) {
        if ROOT_INODE.link_num(inode.ino() as u32, inode.blck_off()) == 1 {
            // clear data if only one link exists
            inode.clear();
        }
        return ROOT_INODE.unlink(path.as_str());
    }
    ROOT_INODE.unlink(path.as_str())
}