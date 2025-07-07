mod attributes;
mod class_file_parser;
mod class_reader;
mod constant_pool;
mod util;

use std::{
    fs,
    io::{self, Read},
};

use bpaf::{Bpaf, Parser};

use crate::{
    class_file_parser::{ClassParser, InstanceKlass, ParserContext},
    class_reader::ClassReader,
};

#[derive(Bpaf)]
/// 解析*.class文件并生成ast
struct Args {
    /// 标准输入和文件输入
    #[bpaf(positional("FILE"), optional)]
    file: Option<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = args().run();
    let file = resolve_stdin(&args.file);
    let class_reader = ClassReader::from(file);
    let mut parse_ctx = ParserContext::new(class_reader);
    let klass = <InstanceKlass as ClassParser>::parse(&mut parse_ctx)?;
    println!("{:?}", klass);
    Ok(())
}

fn resolve_stdin(file: &Option<String>) -> String {
    match file {
        Some(path) => fs::read_to_string(path).unwrap(),
        None => {
            let mut buf = String::new();
            io::stdin().read_to_string(&mut buf).unwrap();
            buf
        }
    }
}

#[cfg(test)]
mod tests {
    use rust_embed::RustEmbed;

    use crate::{
        class_file_parser::{ClassParser, InstanceKlass, ParserContext},
        class_reader::ClassReader,
    };

    #[derive(RustEmbed)]
    #[folder = "asset/"]
    #[exclude("*.java")]
    struct TestContext;

    impl TestContext {
        pub fn parse_class_file(path: &str) {
            let file = Self::get(path).unwrap();
            let string = ClassReader::from(file.data.as_ref().to_vec());
            let class_reader = string;
            let mut parser_ctx = ParserContext::new(class_reader);
            <InstanceKlass as ClassParser>::parse(&mut parser_ctx).unwrap();
        }
    }

    #[test]
    fn test_code_attribute() {
        TestContext::parse_class_file("Simple1Impl.class");
    }
}
