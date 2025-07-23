use jrm_macro::native_fn;
use jrm_native::{JClass, JObject, NativeContext};

#[native_fn(class_path = "java.lang.System")]
pub fn setIn0(ctx: &NativeContext, class: JClass, stream: JObject) {
    if let Ok(field) = (ctx.get_static_field)(&class, "java/io/InputStream", "in") {
        (ctx.set_static_object_field)(&class, field, stream);
    }
}

#[native_fn(class_path = "java.lang.System")]
pub fn setOut0(ctx: &NativeContext, class: JClass, stream: JObject) {
    if let Ok(field) = (ctx.get_static_field)(&class, "java/io/InputStream", "in") {
        (ctx.set_static_object_field)(&class, field, stream);
    }
}
