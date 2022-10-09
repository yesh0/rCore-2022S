# Lab6
（好像应该是 Lab5？）

## 实现的功能

死锁检测，算法很简单，但是状态跟踪还挺麻烦的……

跟踪什么状态？
* 对每一个线程：
  - 获取的锁
  - 正在尝试获取的锁
* 对进程：
  - 每个锁的余量

如何更新状态？
* 要保证的：这个锁被获取的量 + 余量 = 总量。
  - 线程正在获取的：在系统调用直接记录即可。
  - 锁余量：修改接口给出余量。
  - 被获取的量：这里的实现只能进到锁内部记录，因为要在调度前把锁所有权转移给记录下来。

用时大概半个下午加一晚上（请见 ch7 branch 最后一个 commit 的时间以及 ch8 的 commit 时间）。

## 问答作业

### 回收资源

- 需要回收的资源有哪些？

  * 除了当前的 kernel stack 其它的都需要回收，只是 Drop trait 以及 RAII 和引用计数让回收可以自动一点。例如：
    * TaskUserRes：因为这里通过 Drop trait 支持了回收单独的线程及其内存（虽然没有用到），所以最后也必须是手动释放，否则 TaskUserRes drop 会释放内存而 MemorySet drop 了也会释放。
    * 文件描述符表以及对应的文件。
    * 进程内存。
    * TaskControlBlock 等 Arc 引用归零自动回收。
    * 各种锁，等 ProcessControlBlock 引用归零顺带回收。
    * ProcessControlBlock 等 waitpid 把引用释放掉再回收。
  * Kernel stack 在更换栈之后，例如在 initproc 里再回收。

- 其他线程的 TaskControlBlock 可能在哪些位置被引用，分别是否需要回收，为什么？

  * `grep -r TaskControlBlock os/src`：其它的有：
    * `src/timer.rs`: 正在睡眠的线程，目前实现看起来没有处理，会崩。最好把整个 timer 回收掉，抑或是调度时检查一下线程是否正常。需要回收。
    * `src/task/manager.rs`: 就序任务队列。怀疑在机缘巧合下也会崩，有点难复现。需要回收。
    * `src/sync/condvar.rs`, `src/sync/mutex.rs`, `src/sync/semaphore.rs`: 等待队列。因为不会再解锁了，所以不回收也可以，等锁被回收（见上）即可。

### 对比实现区别

前者该解锁的时候解了锁，锁转移的时候也解了锁。实现上就是错误的：
- Thread 1 获取锁
- Thread 2 等待锁
- Thread 1 唤醒 Thread 2，但把锁给解锁了
- Thread 3 获取锁成功

## 建议

1. 回答问答题时满地找 Drop 其实有点累，不知道会不会有更逻辑统一的资源管理方法……
   另外 TaskUserRes 的双重释放也许会暗示着资源所有权可以再梳理一下？
   （这个人之前完全没有用过 Rust，都是乱说的）

2. TaskControlBlock 的回收的确有点问题，下面是 timer 相关出错的一个复现：

   ```rust
   fn oversleeper() {
     sleep(1000);
     exit(0);
   }

   #[no_mangle]
   pub fn main() -> i32 {
     thread_create(oversleeper as usize, 0);
     sleep(100);
     0
   }
   ```

   在最开始的 ch8 分支上也会直接 panic：
   ```
   [kernel] Panicked at src/task/processor.rs:109 called `Option::unwrap()` on a `None` value
   ```

## 课程实验收获和改进建议

### 实验收获

- 大概了解了系统调用的较底层实现以及语言标准库与系统调用的包装关系。
- 大概了解了 Rust 编写操作系统相关部分的方法，了解了包括 `no_std` 以及 `alloc` 等的内容。
- 了解了 Rust 交叉编译过程以及 QEMU 结合 GDB debug 的流程。
- 了解了操作系统内存管理的大致思路。
- 了解了基于 stride 的调度算法。

### 改进建议

- 2022S（春季学期）的仓库的编译流程会将用户例程 `cargo clean` 掉，如果可以优化一下编译流程节省一点重新编译的时间会更好。

