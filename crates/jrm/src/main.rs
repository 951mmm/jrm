mod attribute;
mod class_file_parser;
mod class_reader;
mod constant_pool;
mod util;

use crate::class_file_parser::{ClassParser, InstanceKlass, ParserContext};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut class_reader = util::setup()?;
    let mut parse_ctx = ParserContext::new();
    let klass = <InstanceKlass as ClassParser>::parse(&mut class_reader, &mut parse_ctx)?;
    println!("{:?}", klass);
    Ok(())
}
