pub mod attributes;
pub mod class_file_parser;
pub mod class_reader;
pub mod constant_pool;
pub mod instance_klass;

pub use constant_pool::*;

#[cfg(test)]
mod tests {
    use std::{
        env,
        error::Error,
        fs,
        path::{Path, PathBuf},
    };

    use dotenvy::dotenv;
    use rstest::rstest;
    use rust_embed::RustEmbed;

    use crate::instance_klass::InstanceKlass;

    #[derive(RustEmbed)]
    #[folder = "../../asset"]
    struct Asset;

    impl Asset {
        pub fn get_class_bytes(file_name: &str) -> Vec<u8> {
            let msg = &format!("{} not found", file_name);
            let file = Asset::get(&format!("{}.class", file_name)).expect(msg);
            let byte = file.data;
            byte.to_vec()
        }
    }

    #[rstest]
    #[case("Simple1Impl", "simple class impl runnable")]
    #[case("TestAnnotation", "test parse annoatation")]
    fn test_parser(#[case] file_name: &str, #[case] desc: &str) {
        dotenv().ok();
        let closure = || -> Result<(), Box<dyn Error>> {
            let bytes = Asset::get_class_bytes(file_name);
            let instance_klass = InstanceKlass::parse_from_bytes(bytes)?;
            println!("instance_klass is: {:?}", instance_klass);
            Ok(())
        };
        closure().expect(desc);
    }
}
