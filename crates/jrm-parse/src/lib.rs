pub mod attributes;
pub mod class_file_parser;
pub mod class_reader;
pub mod constant_pool;
pub mod instance_klass;

pub use constant_pool::*;

#[cfg(test)]
mod tests {
    use std::{env, fs, path::PathBuf};

    use dotenvy::dotenv;
    use rstest::rstest;

    use crate::instance_klass::InstanceKlass;

    #[rstest]
    #[case("Simple1Impl", "simple class impl runnable")]
    fn test_parser(#[case] file_name: &str, #[case] desc: &str) {
        dotenv().ok();
        let dir_path = env::var("JAVA_CLASS_DIR_PATH").unwrap();
        let class_file_path = PathBuf::from(dir_path).join(format!("{}.class", file_name));
        println!("path is: {}", class_file_path.display());
        let bytes = fs::read(class_file_path).unwrap();
        let instance_klass = InstanceKlass::parse_from_bytes(bytes).unwrap();
        println!("instance_klass is: {:?}", instance_klass);
    }
}
