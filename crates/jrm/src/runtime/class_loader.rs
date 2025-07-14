use std::{
    collections::HashMap,
    fs,
    path::PathBuf,
    sync::{Arc, RwLock},
};

use dashmap::DashMap;

use crate::parse::{
    class_file_parser::ClassParser,
    instance_klass::{self, InstanceKlass},
};

#[derive(Debug, thiserror::Error)]
pub enum ClassLoaderError {
    #[error("class not found: {0}")]
    ClassNotFoundError(String),
    #[error("class parse error: {0}")]
    ClassParseError(#[from] anyhow::Error),
}
pub struct ClassLoader {
    parent: Option<Arc<ClassLoader>>,
    loaded_classes: DashMap<String, Arc<InstanceKlass>>,
    search_paths: Vec<PathBuf>,
}

impl ClassLoader {
    pub fn new(parent: Option<Arc<ClassLoader>>, paths: Vec<impl Into<PathBuf>>) -> Self {
        Self {
            parent,
            loaded_classes: DashMap::new(),
            search_paths: paths.into_iter().map(Into::into).collect(),
        }
    }
    fn find_class(&self, name: &str) -> Option<Arc<InstanceKlass>> {
        if let Some(class) = self.loaded_classes.get(name) {
            return Some(class.clone());
        }
        if let Some(parent) = &self.parent {
            // REVIEW 需要缓存到当前加载器吗
            return parent.find_class(name);
        }

        None
    }
    fn define_class(
        &mut self,
        // FIXME 使用strmap
        name: String,
        instance_klass: Arc<InstanceKlass>,
    ) -> Arc<InstanceKlass> {
        self.loaded_classes
            .entry(name)
            .or_insert(instance_klass)
            .clone()
    }
    fn load_from_path(&self, name: &str) -> Result<Vec<u8>, ClassLoaderError> {
        let path_suffix = name.replace('.', "/") + ".class";
        for base_path in &self.search_paths {
            let full_path = base_path.join(&path_suffix);
            if let Ok(bytes) = fs::read(full_path) {
                return Ok(bytes);
            }
        }
        Err(ClassLoaderError::ClassNotFoundError(name.to_string()))
    }
    pub fn load_class(&mut self, name: String) -> Result<Arc<InstanceKlass>, ClassLoaderError> {
        if let Some(class) = self.find_class(&name) {
            return Ok(class);
        }
        // TODO 默认的bootstrap加载。通过java home
        let bytes = self.load_from_path(&name)?;
        let instance_klass = InstanceKlass::parse(&mut bytes.into())?;
        let instance_klass = Arc::new(instance_klass);
        Ok(self.define_class(name, instance_klass))
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use rstest::{fixture, rstest};

    use crate::runtime::class_loader::ClassLoader;

    #[fixture]
    fn class_loader() -> ClassLoader {
        let paths = vec!["/home/ww/Documents/note/jrm/crates/jrm/asset"];
        let class_loader = ClassLoader::new(None, paths);
        class_loader
    }

    #[rstest]
    fn test_load_simpl1impl(mut class_loader: ClassLoader) -> Result<(), Box<dyn Error>> {
        let class = class_loader.load_class("Simple1Impl".to_string())?;
        Ok(())
    }
}
