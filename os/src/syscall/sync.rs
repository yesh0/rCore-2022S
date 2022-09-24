use crate::sync::{Condvar, Mutex, MutexBlocking, MutexSpin, Semaphore};
use crate::task::{
    block_current_and_run_next, current_process, current_task, Resource, ResourceType,
    TaskControlBlock,
};
use crate::timer::{add_timer, get_time_ms};
use alloc::sync::Arc;
use alloc::vec::Vec;

use super::thread::sys_gettid;

pub fn sys_sleep(ms: usize) -> isize {
    let expire_ms = get_time_ms() + ms;
    let task = current_task().unwrap();
    add_timer(expire_ms, task);
    block_current_and_run_next();
    0
}

// LAB5 HINT: you might need to maintain data structures used for deadlock detection
// during sys_mutex_* and sys_semaphore_* syscalls
pub fn sys_mutex_create(blocking: bool) -> isize {
    let process = current_process();
    let mutex: Option<Arc<dyn Mutex>> = if !blocking {
        Some(Arc::new(MutexSpin::new()))
    } else {
        Some(Arc::new(MutexBlocking::new()))
    };
    let mut process_inner = process.inner_exclusive_access();
    if let Some(id) = process_inner
        .mutex_list
        .iter()
        .enumerate()
        .find(|(_, item)| item.is_none())
        .map(|(id, _)| id)
    {
        process_inner.mutex_list[id] = mutex;
        id as isize
    } else {
        process_inner.mutex_list.push(mutex);
        process_inner.mutex_list.len() as isize - 1
    }
}

fn set_need_resource(type_: ResourceType, id: usize) {
    let task = current_task().unwrap();
    let mut inner = task.inner_exclusive_access();
    assert!(inner.need.replace(Resource { id, type_ }).is_none());
}

fn mark_resource_released(type_: ResourceType, id: usize) {
    let task = current_task().unwrap();
    let mut inner = task.inner_exclusive_access();
    if let Some(pos) = inner
        .resources
        .iter()
        .position(|res| res.type_ == type_ && id == res.id)
    {
        inner.resources.swap_remove(pos);
    }
}

fn deadlock_detected(
    tasks: &Vec<Option<Arc<TaskControlBlock>>>,
    mutex_list: &Vec<Option<Arc<dyn Mutex>>>,
    semaphore_list: &Vec<Option<Arc<Semaphore>>>,
    type_: ResourceType,
    id: usize,
) -> bool {
    let len = mutex_list.len() + semaphore_list.len();

    let mut allocation = Vec::<u32>::new();
    let mut need = Vec::<u32>::new();
    allocation.resize(len * tasks.len(), 0);
    need.resize(len * tasks.len(), 0);

    let mut work = Vec::<u32>::new();
    work.resize(len, 0);

    // Init the matrices
    let get_id = |id: usize, type_: ResourceType| match type_ {
        ResourceType::Semaphore => id + mutex_list.len(),
        ResourceType::Mutex => id,
    };
    for (tid, option) in tasks.iter().enumerate() {
        if let Some(task) = option {
            let inner = task.inner_exclusive_access();
            for res in &inner.resources {
                allocation[get_id(res.id, res.type_) + len * tid] += 1;
            }
            if let Some(res) = &inner.need {
                need[get_id(res.id, res.type_) + len * tid] += 1;
            }
        }
    }
    need[get_id(id, type_) + len * sys_gettid() as usize] += 1;

    // Init work
    for (i, option) in mutex_list.iter().enumerate() {
        if let Some(mutex) = option {
            if !mutex.is_locked() {
                work[i] += 1;
            }
        }
    }
    for (i, option) in semaphore_list.iter().enumerate() {
        if let Some(semaphore) = option {
            let count = semaphore.inner.exclusive_access().count;
            work[i + mutex_list.len()] += if count > 0 { count } else { 0 } as u32;
        }
    }

    // Finish
    let mut finish = Vec::<bool>::new();
    finish.resize(tasks.len(), false);

    loop {
        let task = finish.iter().enumerate().find(|(tid, finished)| {
            if **finished {
                false
            } else {
                for j in 0..len {
                    if need[tid * len + j] > work[j] {
                        return false;
                    }
                }
                true
            }
        });
        if let Some((tid, _)) = task {
            finish[tid] = true;
            for j in 0..len {
                work[j] += allocation[tid * len + j];
            }
        } else {
            break;
        }
    }

    finish.contains(&false)
}

// LAB5 HINT: Return -0xDEAD if deadlock is detected
pub fn sys_mutex_lock(mutex_id: usize) -> isize {
    let process = current_process();
    let process_inner = process.inner_exclusive_access();
    if process_inner.deadlock_detect
        && deadlock_detected(
            &process_inner.tasks,
            &process_inner.mutex_list,
            &process_inner.semaphore_list,
            ResourceType::Mutex,
            mutex_id,
        )
    {
        return -0xDEAD;
    }
    let mutex = Arc::clone(process_inner.mutex_list[mutex_id].as_ref().unwrap());
    drop(process_inner);
    drop(process);
    set_need_resource(ResourceType::Mutex, mutex_id);
    mutex.lock();
    0
}

pub fn sys_mutex_unlock(mutex_id: usize) -> isize {
    let process = current_process();
    let process_inner = process.inner_exclusive_access();
    let mutex = Arc::clone(process_inner.mutex_list[mutex_id].as_ref().unwrap());
    drop(process_inner);
    drop(process);
    mark_resource_released(ResourceType::Mutex, mutex_id);
    mutex.unlock();
    0
}

pub fn sys_semaphore_create(res_count: usize) -> isize {
    let process = current_process();
    let mut process_inner = process.inner_exclusive_access();
    let id = if let Some(id) = process_inner
        .semaphore_list
        .iter()
        .enumerate()
        .find(|(_, item)| item.is_none())
        .map(|(id, _)| id)
    {
        process_inner.semaphore_list[id] = Some(Arc::new(Semaphore::new(res_count)));
        id
    } else {
        process_inner
            .semaphore_list
            .push(Some(Arc::new(Semaphore::new(res_count))));
        process_inner.semaphore_list.len() - 1
    };
    id as isize
}

pub fn sys_semaphore_up(sem_id: usize) -> isize {
    let process = current_process();
    let process_inner = process.inner_exclusive_access();
    let sem = Arc::clone(process_inner.semaphore_list[sem_id].as_ref().unwrap());
    drop(process_inner);
    mark_resource_released(ResourceType::Semaphore, sem_id);
    sem.up();
    0
}

// LAB5 HINT: Return -0xDEAD if deadlock is detected
pub fn sys_semaphore_down(sem_id: usize) -> isize {
    let process = current_process();
    let process_inner = process.inner_exclusive_access();
    if process_inner.deadlock_detect
        && deadlock_detected(
            &process_inner.tasks,
            &process_inner.mutex_list,
            &process_inner.semaphore_list,
            ResourceType::Semaphore,
            sem_id,
        )
    {
        return -0xDEAD;
    }
    let sem = Arc::clone(process_inner.semaphore_list[sem_id].as_ref().unwrap());
    drop(process_inner);
    set_need_resource(ResourceType::Semaphore, sem_id);
    sem.down();
    0
}

pub fn sys_condvar_create(_arg: usize) -> isize {
    let process = current_process();
    let mut process_inner = process.inner_exclusive_access();
    let id = if let Some(id) = process_inner
        .condvar_list
        .iter()
        .enumerate()
        .find(|(_, item)| item.is_none())
        .map(|(id, _)| id)
    {
        process_inner.condvar_list[id] = Some(Arc::new(Condvar::new()));
        id
    } else {
        process_inner
            .condvar_list
            .push(Some(Arc::new(Condvar::new())));
        process_inner.condvar_list.len() - 1
    };
    id as isize
}

pub fn sys_condvar_signal(condvar_id: usize) -> isize {
    let process = current_process();
    let process_inner = process.inner_exclusive_access();
    let condvar = Arc::clone(process_inner.condvar_list[condvar_id].as_ref().unwrap());
    drop(process_inner);
    condvar.signal();
    0
}

pub fn sys_condvar_wait(condvar_id: usize, mutex_id: usize) -> isize {
    let process = current_process();
    let process_inner = process.inner_exclusive_access();
    let condvar = Arc::clone(process_inner.condvar_list[condvar_id].as_ref().unwrap());
    let mutex = Arc::clone(process_inner.mutex_list[mutex_id].as_ref().unwrap());
    drop(process_inner);
    mark_resource_released(ResourceType::Mutex, mutex_id);
    set_need_resource(ResourceType::Mutex, mutex_id);
    condvar.wait(mutex);
    0
}

// LAB5 YOUR JOB: Implement deadlock detection, but might not all in this syscall
pub fn sys_enable_deadlock_detect(enabled: usize) -> isize {
    let process = current_process();
    let inner = &mut process.inner_exclusive_access();
    match enabled {
        0 => {
            inner.deadlock_detect = false;
            0
        }
        1 => {
            inner.deadlock_detect = true;
            0
        }
        _ => -1,
    }
}
