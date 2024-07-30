pub struct PacketWriter {
    data: Vec<u8>,
}

impl PacketWriter {
    pub fn new() -> Self {
        PacketWriter { data: Vec::new() }
    }

    pub fn write_byte(&mut self, value: u8) {
        self.data.push(value);
    }

    pub fn write_sbyte(&mut self, value: i8) {
        self.data.push(value as u8);
    }

    pub fn write_short(&mut self, value: i16) {
        self.data.extend(&value.to_be_bytes());
    }

    pub fn write_string(&mut self, value: &str) {
        let mut bytes = value.as_bytes().to_vec();
        bytes.resize(64, b' ');
        self.data.extend(&bytes);
    }

    pub fn write_byte_array(&mut self, value: &[u8], size: usize) {
        let mut bytes = value.to_vec();
        bytes.resize(size, 0x00);
        self.data.extend(&bytes);
    }

    pub fn to_bytes(&self) -> &Vec<u8> {
        return &self.data;
    }
    pub fn into_inner(self) -> Vec<u8> {
        self.data
    }
}
