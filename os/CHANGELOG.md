# Absolutely Not A Changelog

（赶工赶完了所以下面大多是回忆性质的）

## Ch6

在 cherry-pick 时遇到了奇怪的 bug：sys_spawn 调用两次会失败或内核 panick，panick 位置印象中是对文件夹的 assertion。
找不到 bug 具体出错，初步怀疑是内存哪里溢出了，把 TaskControlBlock 的数据量调小了，例如中断统计量改为堆分配而不用数组。好了。
但上面的假设也有问题，毕竟我们有 barrier，理论上应该更早一点报错才对。

其它比较简单，毕竟基本只要掉写好的函数就可以了（

## Ch8

主要难点是跟踪状态变化……没有想到比较优雅的写法，直接把一部分代码写到锁的实现里面去了。
