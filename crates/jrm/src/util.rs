use crate::class_reader::ClassReader;
use dotenvy::dotenv;

pub fn setup() -> anyhow::Result<ClassReader> {
    dotenv().ok();
    let path = std::env::var("JAVA_CLASS_PATH").expect("CLASS_PATH not set");
    ClassReader::read_path(path)
}
