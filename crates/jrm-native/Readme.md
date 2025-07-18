# jrm-native
作用类似jni.h或JNI::*

规范native调用的接口一致性

函数名使用LIBNAME_com_examle_class_method保持和java的一致性

函数签名使用java的类型并转为jrm-native支持的类型

为何要保持这种落后的链接方式？因为System.loadLibrary