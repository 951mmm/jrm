use std::{fs::File, io::Read, vec};

pub struct ClassReader {
    buffer: Vec<u8>,
    cur: usize,
}

impl ClassReader {
    pub fn read_path(path: String) -> anyhow::Result<ClassReader> {
        println!("path is: {}", path);
        let class_file = File::open(path)?;
        let file_size = class_file.metadata()?.len() as usize;
        let mut buffer = vec![0; file_size];

        let mut class_file = class_file;
        class_file.read_exact(&mut buffer)?;

        Ok(ClassReader { buffer, cur: 0 })
    }

    pub fn read_one_byte(&mut self) -> Option<u8> {
        let result = self.buffer.get(self.cur).cloned();
        self.cur += 1;
        result
    }

    pub fn read_two_bytes(&mut self) -> Option<u16> {
        let byte1 = self.read_one_byte()?;
        let byte2 = self.read_one_byte()?;
        Some(((byte1 as u16) << 8) | (byte2 as u16))
    }

    pub fn read_four_bytes(&mut self) -> Option<u32> {
        let byte1 = self.read_one_byte()?;
        let byte2 = self.read_one_byte()?;
        let byte3 = self.read_one_byte()?;
        let byte4 = self.read_one_byte()?;
        Some(
            ((byte1 as u32) << 24)
                | ((byte2 as u32) << 16)
                | ((byte3 as u32) << 8)
                | (byte4 as u32),
        )
    }

    pub fn read_bydes(&mut self, size: usize) -> Option<Vec<u8>> {
        if self.cur + size > self.buffer.len() {
            return None;
        }
        let bytes = self.buffer[self.cur..self.cur + size].to_vec();
        self.cur += size;
        Some(bytes)
    }
}
