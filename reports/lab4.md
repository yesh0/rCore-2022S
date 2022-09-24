# Lab4

## 实现的功能

1. Cherry-pick 了前几章的内容。
2. Link / Unlink / Fstat 需要对文件的引用数作记录，修改了 Inode 结构加入了相应内容。
   因为想要保持 Inode 的 32 字节大小，所以选择了把引用计数和文件类型压缩成一个 u32 field。
3. Fstat 需要 Inode ID，但是我们没有记录对应内容，这里直接用偏移量算回 Inode ID。
   另外因为一个文件描述符可能对应物理文件或是例如 stdio 里的，这里直接在 File trait 里加了 stat。
4. Link 就是两个目录项指向同一个 Inode，对应引用数增加即可。
5. Unlink 的话把目录项删除，然后在引用数归零时删除 Inode。

## 问答作业

### 在我们的easy-fs中，root inode起着什么作用？如果root inode中的内容损坏了，会发生什么？

root inode 就是根目录对应的 inode，所有的文件名的搜索都要经过 root inode -> root inode 内容 -> 遍历目录项 -> 找到对应文件名的 inode 这样的过程。

损坏了也就是上面的过程从最开始就行不通了，相当于整个分区不可用了。
