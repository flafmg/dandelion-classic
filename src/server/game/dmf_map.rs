use std::{
    fs::File,
    io::{self, ErrorKind, Read, Write},
};

const HEADER_INDENTIFIER: &str = "DANDELION MAP FORMAT";
const HEADER_VERSION: u8 = 0x00;

#[derive(Debug, Clone)]
pub struct DmfMap {
    pub x_spawn: i16,
    pub y_spawn: i16,
    pub z_spawn: i16,

    pub x_size: i16,
    pub y_size: i16,
    pub z_size: i16,

    pub blocks: Vec<u8>,
}

impl DmfMap {
    pub fn new(
        x_spawn: i16,
        y_spawn: i16,
        z_spawn: i16,
        x_size: i16,
        y_size: i16,
        z_size: i16,
    ) -> Self {
        let total_blocks = (x_size as usize * y_size as usize * z_size as usize);
        return Self {
            x_spawn,
            y_spawn,
            z_spawn,

            x_size,
            y_size,
            z_size,

            blocks: vec![0x00; total_blocks],
        };
    }
    pub fn get_block(&self, x: i16, y: i16, z: i16) -> u8 {
        if x < self.x_size && y < self.y_size && z < self.z_size {
            let index = (y as usize * self.z_size as usize * self.x_size as usize)
                + (z as usize * self.x_size as usize)
                + x as usize;
            self.blocks[index]
        } else {
            println!(
                "Error: attempted to get block outside world ({}, {}, {})",
                x, y, z
            );
            0x00
        }
    }
    pub fn set_block(&mut self, x: i16, y: i16, z: i16, block: u8) {
        if x < self.x_size && y < self.y_size && z < self.z_size {
            let index = (y as usize * self.z_size as usize * self.x_size as usize)
                + (z as usize * self.x_size as usize)
                + x as usize;
            self.blocks[index] = block;
        } else {
            println!(
                "Error: attempted to place block outside world ({}, {}, {})",
                x, y, z
            );
        }
    }
    pub fn set_spawn_point(&mut self, x: i16, y: i16, z: i16) {
        self.x_spawn = x;
        self.y_spawn = y;
        self.z_spawn = z;
    }
    pub fn save_file(&self, path: &str) -> io::Result<()> {
        let mut file = File::create(path)?;
        print!("saving file to {}", path);

        file.write_all(HEADER_INDENTIFIER.as_bytes())?;
        file.write_all(&[HEADER_VERSION])?;

        file.write_all(&self.x_spawn.to_le_bytes())?;
        file.write_all(&self.y_spawn.to_le_bytes())?;
        file.write_all(&self.z_spawn.to_le_bytes())?;

        file.write_all(&self.x_size.to_le_bytes())?;
        file.write_all(&self.y_size.to_le_bytes())?;
        file.write_all(&self.z_size.to_le_bytes())?;

        file.write_all(&self.blocks)?;
        file.flush();

        return Ok(());
    }

    pub fn load_file(path: &str) -> io::Result<Self> {
        let mut file = File::open(&path)?;

        let mut indentifier = [0u8; 20];
        file.read_exact(&mut indentifier)?;
        if &indentifier != HEADER_INDENTIFIER.as_bytes() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid header identifier",
            ));
        }

        let mut version = [0u8; 1];
        file.read_exact(&mut version)?;
        if version[0] != HEADER_VERSION {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Unsupported version",
            ));
        }

        let mut x_spawn = [0u8; 2];
        file.read_exact(&mut x_spawn)?;
        let x_spawn = i16::from_le_bytes(x_spawn);

        let mut y_spawn = [0u8; 2];
        file.read_exact(&mut y_spawn)?;
        let y_spawn = i16::from_le_bytes(y_spawn);

        let mut z_spawn = [0u8; 2];
        file.read_exact(&mut z_spawn)?;
        let z_spawn = i16::from_le_bytes(z_spawn);

        let mut x_size = [0u8; 2];
        file.read_exact(&mut x_size)?;
        let x_size = i16::from_le_bytes(x_size);

        let mut y_size = [0u8; 2];
        file.read_exact(&mut y_size)?;
        let y_size = i16::from_le_bytes(y_size);

        let mut z_size = [0u8; 2];
        file.read_exact(&mut z_size)?;
        let z_size = i16::from_le_bytes(z_size);

        let total_blocks = (x_size as usize * y_size as usize * z_size as usize);
        let mut blocks = vec![0u8; total_blocks];
        file.read_exact(&mut blocks)?;

        return Ok(Self {
            x_spawn,
            y_spawn,
            z_spawn,

            x_size,
            y_size,
            z_size,

            blocks,
        });
    }
}
