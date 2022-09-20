# Lab3

## 实现的功能

1. 把 Lab1、Lab2 的搬过来了，没有太大变化。
   - `sys_get_time`
   - `sys_task_info`
   - `sys_mmap`
   - `sys_munmap`
2. 实现 `sys_spawn`：参考 init 进程创建的流程，然后再记录进程的 `parent` 加入 `children` 等。
3. Stride 调度并实现 `sys_set_priority`：
   - 按提示来，只要知道调度的主体部分在 `processor.rs` 的 `run_tasks`，之后要做的就只有修改 `fetch_task` 以及增加 `stride` 而已了。

## 问答作业

### 实际情况是轮到 p1 执行吗？为什么？

   不是，因为溢出了。`250 + 10 => 4`，则此时 stride 最小仍为 p2。

### 为什么？尝试简单说明（不要求严格证明）。

   考虑使极差最大化的情形，此时一定是所有进程 stride 相等且调度选中了优先级最高的进程，
   调度后 STRIDE_MAX – STRIDE_MIN = BigStride / min{prio} <= BigStride / 2。

   反过来若存在 STRIDE_MAX – STRIDE_MIN = BigStride / 2 + N > BigStride / 2 的情形，
   则一定有某次调度没有调度到 stride 最小的进程。

### 补全代码

```rust
impl PartialOrd for Stride {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(match self.0.wrapping_sub(other.0) {
      0 => Ordering::Equal,
      (1..=BigStride/2) => Ordering::Greater,
      _ => Ordering::Less,
    })
  }
}
```

## 建议

1. stride 的测例标准可能太宽了？还没开始实现的时候已经显示通过了……

2. 另外 Tutorial Book 的建议不知道要不要写在这里？

   [Rust 中的动态内存分配](http://rcore-os.cn/rCore-Tutorial-Book-v3/chapter4/1rust-dynamic-allocation.html)：
   “Python/Java 通过引用计数 (Reference Counting) 对所有的对象进行运行时的动态管理。”

   一般现在的垃圾回收都不会用引用计数，至少不是这个术语。所以从严谨性上来讲这部分还是改一改为好。
   （例如 Java 的 ZGC 标记对象用染色指针什么的，会标记有没有，但不会计数。
   引用计数指向很明确是另外一类技术。）
