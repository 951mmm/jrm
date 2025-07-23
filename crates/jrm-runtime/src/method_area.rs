use jimage_rs::JImage;
use std::{collections::HashMap, sync::Arc};

use crate::class::Class;

/// 管理类加载，缓存已加载的类
pub struct MethodArea {
    jimage: JImage,
    loaded_class: HashMap<String, Arc<Class>>,
}

impl MethodArea {
    #[cfg(test)]
    pub fn new() -> Self {
        let java_home = env!("JAVA_HOME");
        let module_path = format!("{}/lib/modules", java_home);
        Self {
            jimage: JImage::open(module_path).unwrap(),
            loaded_class: Default::default(),
        }
    }
}
