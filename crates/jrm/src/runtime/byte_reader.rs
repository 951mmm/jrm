pub trait ByteReader {
    fn read_u1(&self) -> u8;
    fn read_u2(&self) -> u16;
}
