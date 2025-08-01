mod attributes;
mod class_file_parser;
mod class_reader;
mod constant_pool;
mod instance_klass;
mod runtime;
mod test_context;
mod util;

use std::{
    fs,
    io::{self, Read},
};

use bpaf::{Bpaf, Parser};

use crate::{
    class_file_parser::{ClassParser, ParserContext},
    class_reader::ClassReader,
    instance_klass::InstanceKlass,
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
    use crate::test_context::TestContext;
    #[test]
    fn test_code_attribute() {
        let _ = TestContext::parse_class_file("Simple1Impl.class");
    }
}
