//! Process management syscalls

use crate::config::MAX_SYSCALL_NUM;
use crate::mm::translated_byte_buffer;
use crate::task::{exit_current_and_run_next, suspend_current_and_run_next, TaskStatus, sys_call_stat, current_user_token};
use crate::timer::get_time_us;

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

#[derive(Clone, Copy)]
pub struct TaskInfo {
    pub status: TaskStatus,
    pub syscall_times: [u32; MAX_SYSCALL_NUM],
    pub time: usize,
}

pub fn sys_exit(exit_code: i32) -> ! {
    info!("[kernel] Application exited with code {}", exit_code);
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    suspend_current_and_run_next();
    0
}

fn write_to_user_buffer(buffer: &[u8], ptr: *mut u8) {
    let dsts = translated_byte_buffer(current_user_token(), ptr, buffer.len());
    let mut i = 0usize;
    for dst in dsts {
        let slice = &buffer[i .. dst.len()];
        dst.copy_from_slice(slice);
        i += dst.len();
    }
}

fn write_to_user_ptr<T>(t: T, ptr: *mut T) {
    let content = unsafe {
        core::slice::from_raw_parts(&t as *const T as *const u8, core::mem::size_of::<T>())
    };
    write_to_user_buffer(content, ptr as *mut u8);
}

/// stores time info into the supplied pointer
pub fn sys_get_time(ts: *mut TimeVal, _tz: usize) -> isize {
    let us = get_time_us();
    write_to_user_ptr(TimeVal {
        sec: us / 1_000_000,
        usec: us % 1_000_000,
    }, ts);
    0
}

// CLUE: 从 ch4 开始不再对调度算法进行测试~
pub fn sys_set_priority(_prio: isize) -> isize {
    -1
}

// YOUR JOB: 扩展内核以实现 sys_mmap 和 sys_munmap
pub fn sys_mmap(_start: usize, _len: usize, _port: usize) -> isize {
    -1
}

pub fn sys_munmap(_start: usize, _len: usize) -> isize {
    -1
}

/// stores task info into the supplied pointer
pub fn sys_task_info(ti: *mut TaskInfo) -> isize {
    let stat = sys_call_stat();
    write_to_user_ptr(TaskInfo {
        status: TaskStatus::Running,
        syscall_times: stat.sys_call_stat,
        time: (get_time_us() - stat.first_run_time) / 1000,
    }, ti);
    0
}
