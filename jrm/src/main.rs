mod class_file_parser;
mod class_reader;
mod constant_pool;
mod util;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut class_reader = util::setup()?;
    let klass = class_file_parser::ClassFileParser::parse(&mut class_reader)?;
    println!("{:?}", klass);
    Ok(())
}
