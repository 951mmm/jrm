use rust_embed::RustEmbed;

use crate::parse::{
    class_file_parser::{ClassParser, ParserContext},
    class_reader::ClassReader,
    instance_klass::InstanceKlass,
};

#[cfg(test)]
#[derive(RustEmbed)]
#[folder = "asset/"]
#[exclude("*.java")]
pub struct TestContext;

#[cfg(test)]
impl TestContext {
    pub fn parse_class_file(path: &str) -> InstanceKlass {
        let file = Self::get(path).unwrap();
        let string = ClassReader::from(file.data.as_ref().to_vec());
        let class_reader = string;
        let mut parser_ctx = ParserContext::new(class_reader);
        let instance_klass = <InstanceKlass as ClassParser>::parse(&mut parser_ctx).unwrap();
        return instance_klass;
    }
}
