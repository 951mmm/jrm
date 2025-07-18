# libjava
编译成动态库供jvm使用

保存了java.base的所有native调用

使用native接口写法

为何要使用c的动态链接，workspace直接链接不行？

因为java有一个System.loadLibrary()
- [ ] java.lang.string