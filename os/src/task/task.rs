//! Types related to task management

use super::TaskContext;
use crate::task::MAX_SYSCALL_NUM;

#[derive(Copy, Clone)]
/// task stats
pub struct TaskStatistics {
    pub sys_call_stat: [u32; MAX_SYSCALL_NUM],
    pub first_run_time: usize,
}

impl TaskStatistics {
    pub fn zero_init() -> TaskStatistics {
        TaskStatistics { sys_call_stat: [0; MAX_SYSCALL_NUM], first_run_time: 0 }
    }
}

#[derive(Copy, Clone)]
/// task control block structure
pub struct TaskControlBlock {
    pub task_status: TaskStatus,
    pub task_cx: TaskContext,
    pub task_statistics: TaskStatistics,
}

#[derive(Copy, Clone, PartialEq)]
/// task status: UnInit, Ready, Running, Exited
pub enum TaskStatus {
    UnInit,
    Ready,
    Running,
    Exited,
}
