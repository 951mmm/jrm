use jimage_rs::JImage;
use jrm_parse::instance_klass::InstanceKlass;
use std::{borrow::Cow, collections::HashMap, fs, path::Path, sync::Arc};

use crate::{Error, Result, class::Class};
/// 管理类加载，缓存已加载的类
pub struct MethodArea {
    jimage: JImage,
    loaded_classes: HashMap<String, Arc<Class>>,
    // TODO 支持--class-path
}

macro_rules! map_err {
    ($expr: expr) => {
        $expr.map_err(|e| Error::ClassLoaderError(e.into()))?
    };
}

macro_rules! ok_or {
    ($expr: expr, $($args: expr),+) => {
        $expr.ok_or(Error::ClassLoaderError(anyhow::anyhow!($($args),+)))?
    };
}

impl MethodArea {
    pub fn new(java_home: &Path) -> Result<Self> {
        let jimage = map_err!(JImage::open(java_home.join("lib").join("modules")));

        Ok(Self {
            jimage,
            loaded_classes: Default::default(),
        })
    }

    pub fn load_class(&mut self, class_name: &str) -> Result<Arc<Class>> {
        if let Some(class) = self.loaded_classes.get(class_name) {
            return Ok(class.clone());
        }

        let class = self.parse_class_file(class_name)?;
        let class = Arc::new(class);
        self.loaded_classes
            .insert(class_name.to_string(), class.clone());
        Ok(class)
    }

    fn parse_class_file(&self, class_name: &str) -> Result<Class> {
        // 加载bootstrap类
        if let Some(bytes) = self.load_bootstrap_class_bytes(class_name)? {
            let instance_klass = InstanceKlass::parse_from_bytes(bytes.to_vec())?;
            return helper::build_class(class_name, instance_klass);
        }
        // 当前目录加载类
        let class_file_path = Path::new(class_name).join(".class");
        let bytes = map_err!(fs::read(&class_file_path));
        let instance_klass = map_err!(InstanceKlass::parse_from_bytes(bytes));
        helper::build_class(class_name, instance_klass)
    }

    fn load_bootstrap_class_bytes(&self, class_name: &str) -> Result<Option<Cow<'_, [u8]>>> {
        let bootstrap_class_name = format!("/java.base/{}.class", class_name);
        if let Some(bytes) = map_err!(self.jimage.find_resource(&bootstrap_class_name)) {
            Ok(Some(bytes))
        } else {
            Ok(None)
        }
    }
}

mod helper {
    use std::{collections::HashMap, sync::Arc};

    use jrm_parse::{
        ConstantPool,
        attributes::Exception as RawException,
        instance_klass::{InstanceKlass, Method as RawMethod, MethodAccessFlags},
    };

    use crate::{
        Error, Result,
        class::{Class, ClassBuilder},
        heap::ObjectRef,
        helper::TryGetConstant,
        method::{CatchType, Exception, Method, MethodBuilder, MethodId, MethodSignature},
    };

    pub fn build_class(class_name: &str, instance_class: InstanceKlass) -> Result<Class> {
        let constant_pool = instance_class.get_constant_pool();

        let super_class_index = constant_pool
            .try_get(instance_class.get_super_class())?
            .parse_class()?
            .get_name_index();
        let super_class_name = constant_pool.get_utf8_string(super_class_index);

        let interfaces = instance_class.get_interfaces();
        let interface_names =
            interfaces
                .iter()
                .try_rfold::<_, _, Result<Vec<_>>>(vec![], |mut acc, interface| {
                    let interface_index = constant_pool
                        .try_get(interface.get_name_index())?
                        .parse_class()?
                        .get_name_index();
                    let interface_name = constant_pool.get_utf8_string(interface_index);
                    acc.push(interface_name);
                    Ok(acc)
                })?;

        let methods = instance_class.get_methods();

        let methods = build_methods(class_name, methods, constant_pool)?;

        let class = ClassBuilder::default()
            .class_name(class_name.to_string())
            .super_class_name(super_class_name)
            .interface_names(interface_names)
            .build()?;
        Ok(class)
    }

    pub fn build_methods(
        class_name: &str,
        raw_methods: &[RawMethod],
        constant_pool: &Arc<ConstantPool>,
    ) -> Result<HashMap<MethodId, Method>> {
        let mut methods = HashMap::new();
        for raw_method in raw_methods {
            let access_flags = raw_method.get_access_flags();
            let is_native = access_flags.contains(MethodAccessFlags::NATIVE);

            let name_index = raw_method.get_name_index();
            let name = constant_pool.get_utf8_string(name_index);

            let descriptor_index = raw_method.get_descriptor_index();
            // TODO 细化error
            let descriptor = constant_pool.get_utf8_string(descriptor_index);
            let signature = MethodSignature::try_from(descriptor)?;

            let mut method_id = match name.as_str() {
                "<init>" => MethodId::Init(signature.clone()),
                _ => MethodId::Common {
                    name: name.clone(),
                    signature: signature.clone(),
                },
            };

            let attributes = raw_method.get_attributes();

            let mut max_locals = None;
            let mut max_stack = None;
            let mut code = None;
            let mut exception_table = vec![];
            let mut exception_indices = vec![];
            let mut line_numbers = None;
            for attribute in attributes {
                if let Ok(code_attr) = attribute.parse_code() {
                    max_locals = Some(code_attr.get_max_locals());
                    max_stack = Some(code_attr.get_max_stack());
                    code = Some(code_attr.get_code().clone());
                    for raw_exception in code_attr.get_exception_table() {
                        let exception = build_exception(raw_exception, constant_pool)?;
                        exception_table.push(exception);
                    }
                    let code_inner_attributes = code_attr.get_attributes();
                    for attribute in code_inner_attributes {
                        if let Ok(line_number_attr) = attribute.parse_line_number_table() {
                            let line_numbers_temp: HashMap<_, _> = line_number_attr
                                .get_line_number_table()
                                .iter()
                                .map(|line_number| {
                                    (line_number.get_start_pc(), line_number.get_line_number())
                                })
                                .collect();
                            line_numbers = Some(line_numbers_temp);
                        }
                    }
                }
                if let Ok(exception_attr) = attribute.parse_exception() {
                    for exception_class in exception_attr.get_expcetion_index_table() {
                        let class_name_index = exception_class.get_name_index();
                        let class_name = constant_pool.get_utf8_string(class_name_index);
                        exception_indices.push(class_name);
                    }
                }
                if let Ok(runtime_vis_ann_attr) = attribute.parse_runtime_visible_annotations() {
                    let is_handled = runtime_vis_ann_attr.get_annotations().iter().any(|ann| {
                        let type_index = ann.get_type_index();
                        let type_string = constant_pool.get_utf8_string(type_index);
                        type_string == "Ljava/lang/invoke/MethodHandle$PolymorphicSignature;"
                    });
                    if is_handled && is_native {
                        method_id = MethodId::Polymorphic(signature.clone());
                    }
                }
            }
            // TODO reflection method
            let reflection_ref = ObjectRef::null();

            let class_name = Arc::new(class_name.to_string());

            let id = Arc::new(method_id.clone());
            let code = Arc::new(ok_or!(
                code,
                "failed to resolve method {}, invalid code",
                name
            ));
            let method = MethodBuilder::default()
                .class_name(class_name)
                .access_flags(*access_flags)
                .is_native(is_native)
                .name(name.clone())
                .id(id)
                .max_locals(ok_or!(
                    max_locals,
                    "failed to resolve method: {}, missing max locals",
                    name
                ))
                .max_stack(ok_or!(
                    max_stack,
                    "failed to resolve method: {}, missing max stack",
                    name
                ))
                .code(code)
                .exception_table(exception_table)
                .exception_indices(exception_indices)
                .line_numbers(Arc::new(ok_or!(
                    line_numbers,
                    "failed to resolve method: {}, missing line numbers",
                    name
                )))
                .refection_ref(reflection_ref)
                .build()?;

            methods.insert(method_id, method);
        }
        Ok(methods)
    }

    fn build_exception(
        raw_exception: &RawException,
        constant_pool: &Arc<ConstantPool>,
    ) -> Result<Exception> {
        let catch_type_index = raw_exception.get_catch_type();
        let catch_type = if catch_type_index == 0 {
            CatchType::Throwable
        } else {
            let class_name_index = constant_pool
                .try_get(catch_type_index)?
                .parse_class()?
                .get_name_index();
            let class_name = constant_pool.get_utf8_string(class_name_index);
            CatchType::Exception(class_name)
        };
        let exception = Exception::new(
            raw_exception.get_start_pc(),
            raw_exception.get_end_pc(),
            raw_exception.get_handler_pc(),
            catch_type,
        );
        Ok(exception)
    }
}
