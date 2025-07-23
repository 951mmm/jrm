use crate::Result;
pub trait ByteReader {
    fn read_u1(&self) -> Result<u8>;
    fn read_u2(&self) -> Result<u16>;
}
