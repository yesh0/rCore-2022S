# Lab1

## 实现的功能

1. 给 `TaskManager` 里的任务加了统计信息，在初次执行时初始化，提供访问与更新数据的接口。
2. 在系统调用分发处更新统计信息。
3. 实现了 `sys_task_info` 的系统调用，从 `TaskManager` 获取信息复制到给定指针。未对指针做有效性核验。

## 简答作业

### 1. Violation 测试

使用 SBI 版本：

```
RustSBI version 0.2.0-alpha.4
Implementation: RustSBI-QEMU Version 0.0.1
```

报错信息：

1. `ch2b_bad_address.rs`:

   ```
   [ERROR] [kernel] PageFault in application, bad addr = 0x0, bad instruction = 0x804003fa, core dumped.
   ```
   
2. `ch2b_bad_register.rs`:

   ```
   [ERROR] [kernel] IllegalInstruction in application, core dumped.
   ```

3. `ch2b_bad_instructions.rs`:

   ```
   [ERROR] [kernel] IllegalInstruction in application, core dumped.
   ```

### 2. `trap.S` 理解

1. `a0` 代表了内核栈顶的位置，具体来说是当前任务对应的那一个内核栈的栈顶位置。
   `__restore` 使用方法：

   1. 进入陷阱后，从 `__alltraps` 执行下来，或是从别的任务调度过来。
      总之在 `trap_handler` 返回之后接着执行，恢复对应任务的现场，降至 U mode 归还控制权。
   2. 在任务第一次运行的时候，内核栈顶已被 `__switch` 设好，`TrapContext` 也早已在栈顶了。
      `__restore` 的 `sret` 降至 U mode 进入任务入口。

2. `sscratch` 是用户栈顶位置，最后恢复现场临门一脚 `csrrw sp, sscratch, sp` 回到 `sp`，
   `sepc` 是 `sret` 要用的目标调转位置，
   `sstatus` 记录了要回到的特权级信息。
   `t0` ~ `t2` 只是中转而已。

3. `sp` 又名 `x2`，`tp` 又名 `x4`。前者为了读取栈我们还需要用到，后者的话……

   > 还有 tp(x4) 寄存器，除非我们手动出于一些特殊用途使用它，否则一般也不会被用到。
   
   文档是这么说的，但是听说：
   
   > Yeah, it's used by \__thread / thread_local in pthreads / C++ programs.
   
   所以可能也存一下会好？至少我改了代码存了目前没有影响（

4. `csrrw sp, sscratch, sp` 必须放在读完栈内容并且 pop 掉之后。
   这一步把之前用户的栈顶存回 `sp`，把当前 `sp`（即内核栈顶）放到 `sscratch` 里去。

5. 发生在 `sret`。因为 `sstatus` 里存的是用户态（可能来源于 trap 可能来源于初始化的 `TrapContext`）。

6. 这一步把之前用户的栈顶存入 `sscratch`，把 `sscratch` 里的内核栈顶存回 `sp`。

7. `ecall`，然后就到了 `__alltraps`。（当然能引起 PageFault 和 IllegalInstruction 这些 trap 的也可以就是了。）

## 建议

1. 是不是复制粘贴漏删了点东西……`sys_write` 是之后的实验吗？

   ```
   $ grep -r YOUR src
   src/syscall/fs.rs:// YOUR JOB: 修改 sys_write 使之通过测试
   ```

2. `sys_task_info` 可能可以从验证指针有效性的角度做一点下一章的引入。

3. 本章与上章还有一个改动：KERNEL_STACK 从一个栈变成了一堆栈（一堆/栈），
   可能可以稍微说一下？

4. 简答作业里有些行号好像不太对，可能是第二章里的内容？

