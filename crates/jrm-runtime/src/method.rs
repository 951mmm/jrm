use jrm_parse::instance_klass::MethodAccessFlags;

// TODO 异常处理
#[derive(Debug, Default)]
pub struct Method {
    pub name: String,
    pub descriptor: String,
    pub max_locals: u16,
    pub max_stack: u16,
    pub code: Vec<u8>,
    pub access_flags: MethodAccessFlags,
}

impl Method {
    #[cfg(test)]
    pub fn with_max_stack(max_stack: u16) -> Self {
        Self {
            max_stack,
            ..Default::default()
        }
    }
}
