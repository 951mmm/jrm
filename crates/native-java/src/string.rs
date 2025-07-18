use jrm_macro::native_fn;
use jrm_native::{JObject, NativeContext};

#[native_fn(class_path = "java.lang.String")]
pub fn intern(ctx: &NativeContext, this: JObject) -> JObject {
    (ctx.get_intern_string)(ctx, this)
}
