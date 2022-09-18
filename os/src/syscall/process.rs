//! Process management syscalls

use crate::loader::get_app_data_by_name;
use crate::mm::{translated_refmut, translated_str, translated_byte_buffer};
use crate::task::{
    add_task, current_task, current_user_token, exit_current_and_run_next,
    suspend_current_and_run_next, TaskStatus, sys_call_stat,
};
use crate::timer::get_time_us;
use alloc::sync::Arc;
use crate::config::MAX_SYSCALL_NUM;

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
    debug!("[kernel] Application exited with code {}", exit_code);
    exit_current_and_run_next(exit_code);
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    suspend_current_and_run_next();
    0
}

pub fn sys_getpid() -> isize {
    current_task().unwrap().pid.0 as isize
}

/// Syscall Fork which returns 0 for child process and child_pid for parent process
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

/// Syscall Exec which accepts the elf path
pub fn sys_exec(path: *const u8) -> isize {
    let token = current_user_token();
    let path = translated_str(token, path);
    if let Some(data) = get_app_data_by_name(path.as_str()) {
        let task = current_task().unwrap();
        task.exec(data);
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

// YOUR JOB: 引入虚地址后重写 sys_task_info
pub fn sys_task_info(ti: *mut TaskInfo) -> isize {
    let stat = sys_call_stat();
    write_to_user_ptr(TaskInfo {
        status: TaskStatus::Running,
        syscall_times: stat.sys_call_stat,
        time: (get_time_us() - stat.first_run_time) / 1000,
    }, ti);
    0
}

// YOUR JOB: 实现sys_set_priority，为任务添加优先级
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

//
// YOUR JOB: 实现 sys_spawn 系统调用
// ALERT: 注意在实现 SPAWN 时不需要复制父进程地址空间，SPAWN != FORK + EXEC 
pub fn sys_spawn(_path: *const u8) -> isize {
    -1
}
