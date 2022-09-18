//! Process management syscalls

use crate::config::MAX_SYSCALL_NUM;
use crate::mm::{translated_byte_buffer, VirtPageNum};
use crate::task::{
    allocate_page, current_user_token, deallocate_page, exit_current_and_run_next,
    suspend_current_and_run_next, sys_call_stat, TaskStatus,
};
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
        let slice = &buffer[i..dst.len()];
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
    write_to_user_ptr(
        TimeVal {
            sec: us / 1_000_000,
            usec: us % 1_000_000,
        },
        ts,
    );
    0
}

// CLUE: 从 ch4 开始不再对调度算法进行测试~
pub fn sys_set_priority(_prio: isize) -> isize {
    -1
}

pub fn sys_mmap(start: usize, len: usize, port: usize) -> isize {
    if start & ((1usize << 12) - 1) != 0 || port & !0x7usize != 0 || port == 0 {
        return -1;
    }
    let rwx = [port & 1 != 0, port & 2 != 0, port & 4 != 0];
    for addr in (start..(start + len)).step_by(1 << 12) {
        if !allocate_page(VirtPageNum::from(addr >> 12), rwx) {
            sys_munmap(start, addr - start);
            return -1;
        }
    }
    0
}

pub fn sys_munmap(start: usize, len: usize) -> isize {
    if start & (1usize << 12 - 1) != 0 {
        return -1;
    }
    for addr in (start..(start + len)).step_by(1 << 12) {
        if !deallocate_page(VirtPageNum::from(addr >> 12)) {
            return -1;
        }
    }
    0
}

/// stores task info into the supplied pointer
pub fn sys_task_info(ti: *mut TaskInfo) -> isize {
    let stat = sys_call_stat();
    write_to_user_ptr(
        TaskInfo {
            status: TaskStatus::Running,
            syscall_times: stat.sys_call_stat,
            time: (get_time_us() - stat.first_run_time) / 1000,
        },
        ti,
    );
    0
}
