use std::{
    any,
    fmt::{Debug, Display, Formatter},
};

use anyhow::bail;
use jrm_macro::{ClassFileParse, KlassDebug, parse_not_zero};

use crate::{
    class_reader::ClassReader,
    constant_pool::{Constant, ConstantPool, ConstantUtf8, ConstantWrapper},
};

pub struct ClassFileParser;

impl ClassFileParser {
    fn parse_magic(class_reader: &mut ClassReader) -> anyhow::Result<u32> {
        let magic = class_reader.read_four_bytes();
        match magic {
            Some(m) if m == 0xCAFEBABE => Ok(m),
            _ => bail!("Invalid class file magic number"),
        }
    }

    fn parse_version(class_reader: &mut ClassReader) -> anyhow::Result<(u16, u16)> {
        let minor_version = class_reader.read_two_bytes().unwrap_or(0);
        let major_version = class_reader.read_two_bytes().unwrap_or(0);
        if major_version < 45 || major_version > 70 {
            bail!(
                "Unsupported class file version: {}.{}",
                major_version,
                minor_version
            );
        }
        Ok((minor_version, major_version))
    }

    fn parse_constant_pool(class_reader: &mut ClassReader) -> anyhow::Result<(u16, ConstantPool)> {
        let constant_pool_count = class_reader.read_two_bytes().unwrap_or(0);
        let mut constant_pool = ConstantPool::with_capacity(constant_pool_count as usize);
        constant_pool.add(ConstantWrapper::new_placeholder(0)); // Index 0 is reserved
        for i in 1..constant_pool_count {
            let tag = class_reader.read_one_byte().unwrap();

            let constant_wrapper = match tag {
                1 => {
                    let java_type = "UTF-8";

                    let len =
                        Self::read_constant_bytes(class_reader.read_two_bytes(), java_type, i)?;
                    let utf8_bytes = Self::read_constant_bytes(
                        class_reader.read_bydes(len as usize),
                        java_type,
                        i,
                    )?;
                    ConstantWrapper::new_utf_8(
                        tag,
                        ConstantUtf8 {
                            length: len,
                            bytes: utf8_bytes,
                        },
                    )
                }
                3 => {
                    let integer =
                        Self::read_constant_bytes(class_reader.read_four_bytes(), "Integer", i)?;
                    ConstantWrapper::new_integer(tag, integer as i32)
                }
                4 => {
                    let float = f32::from_bits(Self::read_constant_bytes(
                        class_reader.read_four_bytes(),
                        "Float",
                        i,
                    )?);
                    ConstantWrapper::new_float(tag, float)
                }
                5..7 => {
                    unimplemented!()
                }
                7 => {
                    let class_index =
                        Self::read_constant_bytes(class_reader.read_two_bytes(), "Class", i)?;
                    ConstantWrapper::new_class(tag, class_index)
                }
                8 => {
                    let string_index =
                        Self::read_constant_bytes(class_reader.read_two_bytes(), "String", i)?;
                    ConstantWrapper::new_string(tag, string_index)
                }
                9..12 => Self::parse_address_constant(
                    class_reader,
                    tag,
                    "file, method, Interface, Methodref",
                    i,
                )?,
                12 => Self::parse_address_constant(class_reader, tag, "NameAndType", i)?,
                15 => {
                    let java_type = "MethodHandle";
                    let reference_kind =
                        Self::read_constant_bytes(class_reader.read_one_byte(), java_type, i)?;
                    let reference_index =
                        Self::read_constant_bytes(class_reader.read_two_bytes(), java_type, i)?;
                    ConstantWrapper::new_method_handle(tag, reference_kind, reference_index)
                }
                16 => {
                    let descriptor_index =
                        Self::read_constant_bytes(class_reader.read_two_bytes(), "MethodType", i)?;
                    ConstantWrapper::new_method_type(tag, descriptor_index)
                }
                17 => Self::parse_address_constant(class_reader, tag, "Dynamic", i)?,
                18 => Self::parse_address_constant(class_reader, tag, "InvokeDynamic", i)?,
                19 => {
                    let module_index =
                        Self::read_constant_bytes(class_reader.read_two_bytes(), "Module", i)?;
                    ConstantWrapper::new_module(tag, module_index)
                }
                20 => {
                    let package_index =
                        Self::read_constant_bytes(class_reader.read_two_bytes(), "Package", i)?;
                    ConstantWrapper::new_package(tag, package_index)
                }
                _ => {
                    println!("tag {} is valid", tag);
                    ConstantWrapper::new_placeholder(tag)
                }
            };

            constant_pool.add(constant_wrapper);
        }

        Ok((constant_pool_count, constant_pool))
    }

    fn read_constant_bytes<T>(data: Option<T>, java_type: &str, index: u16) -> anyhow::Result<T> {
        match data {
            Some(data) => Ok(data),
            None => bail!(
                "Failed to read {} index for constant pool entry {}",
                java_type,
                index
            ),
        }
    }

    fn parse_address_constant(
        class_reader: &mut ClassReader,
        tag: u8,
        java_type: &str,
        index: u16,
    ) -> anyhow::Result<ConstantWrapper> {
        let low_byte = Self::read_constant_bytes(class_reader.read_two_bytes(), java_type, index)?;
        let high_byte = Self::read_constant_bytes(class_reader.read_two_bytes(), java_type, index)?;
        let constant = match tag {
            9 => Constant::FieldRef(low_byte, high_byte),
            10 => Constant::MethodRef(low_byte, high_byte),
            11 => Constant::InterfaceMethodRef(low_byte, high_byte),
            12 => Constant::NameAndType(low_byte, high_byte),
            17 => Constant::Dynamic(low_byte, high_byte),
            18 => Constant::InvokeDynamic(low_byte, low_byte),
            _ => {
                unimplemented!();
            }
        };

        Ok(ConstantWrapper { tag, constant })
    }

    fn parse_interfaces(class_reader: &mut ClassReader) -> anyhow::Result<(u16, Vec<u16>)> {
        let interfaces_count = class_reader.read_two_bytes().unwrap_or(0);
        let mut interfaces = Vec::with_capacity(interfaces_count as usize);
        for _ in 0..interfaces_count {
            parse_not_zero!(class_reader, interface_index, 2, "Invalid interface index");
            interfaces.push(interface_index);
        }
        Ok((interfaces_count, interfaces))
    }

    // fn parse_fields(class_reader: &mut ClassReader) -> anyhow::Result<(u16, Vec<Field>)> {
    //     let fields_count = class_reader.read_two_bytes().unwrap_or(0);
    //     let mut fields = Vec::with_capacity(fields_count as usize);
    //     for _ in 0..fields_count {
    //     fields.push(field);
    //     }
    //     Ok((fields_count, fields))
    // }

    fn parse_property(class_reader: &mut ClassReader) -> anyhow::Result<Property> {
        parse_not_zero!(class_reader, access_flags, 2, "Invalid access flags");
        parse_not_zero!(class_reader, name_index, 2, "Invalid name index");
        parse_not_zero!(
            class_reader,
            descriptor_index,
            2,
            "Invalid descriptor index"
        );
        let (attributes_count, attributes) = Self::parse_attributes(class_reader)?;
        Ok(Property {
            access_flags,
            name_index,
            descriptor_index,
            attributes_count,
            attributes,
        })
    }

    fn parse_attributes(class_reader: &mut ClassReader) -> anyhow::Result<(u16, Vec<Attribute>)> {
        let attributes_count = class_reader.read_two_bytes().unwrap_or(0);
        todo!()
    }
}

#[derive(KlassDebug, ClassFileParse)]
pub struct InstanceKlass {
    #[hex]
    magic: u32,
    #[turple_fn(parse_version)]
    minor_version: u16,
    #[turple_fn(parse_version)]
    major_version: u16,
    #[turple_fn(parse_constant_pool)]
    constant_pool_count: u16,
    #[turple_fn(parse_constant_pool)]
    constant_pool: ConstantPool,
    #[hex]
    #[index]
    access_flags: u16,
    #[index]
    this_class: u16,
    #[index]
    super_class: u16,
    #[turple_fn(parse_interfaces)]
    interfaces_count: u16,
    #[turple_fn(parse_interfaces)]
    interfaces: Vec<u16>,
    #[turple_fn(parse_fields)]
    fields_count: u16,
    #[turple_fn(parse_fields)]
    #[property]
    fields: Vec<Field>,
    // methods_count: u16,
    // methods: Vec<MethodInfo>,
    // attributes_count: u16,
    // attributes: Vec<AttributeInfo>,
}

#[derive(Debug)]
pub struct Field(Property);

#[derive(Debug)]
pub struct MetohdInfo(Property);

#[derive(Debug)]
pub struct Property {
    access_flags: u16,
    name_index: u16,
    descriptor_index: u16,
    attributes_count: u16,
    attributes: Vec<Attribute>,
}
#[derive(Debug)]
pub struct Attribute {
    attribute_name_index: u16,
    attribute_length: u32,
    info: Vec<u8>,
}
mod test {
    #[test]
    fn test_parse_class_file() -> anyhow::Result<()> {
        use crate::{class_file_parser::ClassFileParser, util::setup};
        let mut class_reader = setup()?;
        let klass = ClassFileParser::parse(&mut class_reader)?;
        assert_eq!(klass.magic, 0xCAFEBABE);
        assert!(klass.major_version >= 45 && klass.major_version <= 70);
        Ok(())
    }
}
