#[derive(Debug, Clone)]
pub struct PacketReader<'a> {
    data: &'a [u8],
    index: usize,
}

impl<'a> PacketReader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        PacketReader { data, index: 0 }
    }

    pub fn read_byte(&mut self) -> u8 {
        let byte = self.data[self.index];
        self.index += 1;
        return byte;
    }

    pub fn read_sbyte(&mut self) -> i8 {
        let byte = self.data[self.index];
        self.index += 1;
        return byte as i8;
    }

    pub fn read_short(&mut self) -> i16 {
        let bytes = &self.data[self.index..self.index + 2];
        self.index += 2;
        return i16::from_be_bytes([bytes[0], bytes[1]]);
    }

    pub fn read_string(&mut self) -> String {
        let bytes = &self.data[self.index..self.index + 64];
        self.index += 64;
        return String::from_utf8_lossy(bytes).trim_end().to_string();
    }

    pub fn read_byte_array(&mut self, size: usize) -> Vec<u8> {
        let bytes = &self.data[self.index..self.index + size];
        self.index += size;
        return bytes.to_vec();
    }
}
