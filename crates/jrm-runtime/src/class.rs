use derive_builder::Builder;

#[derive(Builder)]
pub struct Class {
    class_name: String,
    super_class_name: String,
    interface_names: Vec<String>,
}
