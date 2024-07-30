// shitty map generation thingy slow as hell
// flat = flat duh
// noise = plain noise
// classic = water in 31 and sand on 33 and bellow

use noise::{NoiseFn, Perlin};

use super::game::dmf_map::DmfMap;

pub struct MapBuilder;

impl MapBuilder {
    pub async fn create_map(preset: &str, params: Option<PresetParams>) -> DmfMap {
        match preset {
            "flat" => Self::create_flat_map(
                params.clone().unwrap().flat_ground_level,
                params.clone().unwrap().dimensions,
            ),
            "noise" => Self::create_noise_map(
                params.clone().unwrap().noise_layers,
                params.clone().unwrap().dimensions,
            ),
            "island" => Self::create_classic_map(
                params.clone().unwrap().noise_layers,
                params.unwrap().dimensions,
            ),
            _ => panic!("Unknown preset"),
        }
    }

    fn create_flat_map(ground_level: u32, dimensions: Dimensions) -> DmfMap {
        let mut map = DmfMap::new(
            0,
            ground_level as i16 + 4,
            0,
            dimensions.x,
            dimensions.y,
            dimensions.z,
        );

        for x in 0..dimensions.x {
            for z in 0..dimensions.z {
                for y in 0..ground_level {
                    map.set_block(x as i16, y as i16, z as i16, 0x01); // stone
                }
                for y in ground_level..(ground_level + 3) {
                    map.set_block(x as i16, y as i16, z as i16, 0x03); // dirt
                }
                map.set_block(x as i16, ground_level as i16 + 3, z as i16, 0x02);
                // grass
            }
        }

        map
    }

    fn create_noise_map(layers: Vec<NoiseLayer>, dimensions: Dimensions) -> DmfMap {
        let mut map = DmfMap::new(0, 64, 0, dimensions.x, dimensions.y, dimensions.z);

        for x in 0..dimensions.x {
            for z in 0..dimensions.z {
                let mut height = 0.0;
                for layer in &layers {
                    let perlin = Perlin::new(50);
                    height +=
                        perlin.get([x as f64 / layer.scale, z as f64 / layer.scale]) * layer.weight;
                }
                let height = (height + dimensions.y as f64 / 2.0).min(dimensions.y as f64) as u32;

                for y in 0..height {
                    map.set_block(x as i16, y as i16, z as i16, 0x01); // stone
                }
                for y in height..(height + 3) {
                    map.set_block(x as i16, y as i16, z as i16, 0x03); // dirt
                }
                map.set_block(x as i16, height as i16 + 3, z as i16, 0x02); // grass
            }
        }

        map
    }

    fn create_classic_map(layers: Vec<NoiseLayer>, dimensions: Dimensions) -> DmfMap {
        const SHORE_FACTOR: f64 = 0.25;
        const SHORE_AFFECTED_DISTANCE: f64 = 25.0; // Distance in blocks that affects the shore

        let water_level = 31;
        let mut map = DmfMap::new(
            0,
            water_level + 1,
            0,
            dimensions.x,
            dimensions.y,
            dimensions.z,
        );

        for x in 0..dimensions.x {
            for z in 0..dimensions.z {
                let mut height = 0.0;
                for layer in &layers {
                    let perlin = Perlin::new(70);
                    height +=
                        perlin.get([x as f64 / layer.scale, z as f64 / layer.scale]) * layer.weight;
                }

                let distance_from_edge_x = (x as f64).min((dimensions.x - x) as f64);
                let distance_from_edge_z = (z as f64).min((dimensions.z - z) as f64);
                let distance_from_edge = distance_from_edge_x.min(distance_from_edge_z);
                let shore_adjustment = if distance_from_edge <= SHORE_AFFECTED_DISTANCE {
                    (1.0 - distance_from_edge / SHORE_AFFECTED_DISTANCE) * SHORE_FACTOR
                } else {
                    0.0
                };

                let height = ((height + dimensions.y as f64 / 2.0) * (1.0 - shore_adjustment))
                    .min(dimensions.y as f64) as u32;

                for y in 0..height {
                    map.set_block(x as i16, y as i16, z as i16, 0x01);
                }
            }
        }

        for x in 0..dimensions.x {
            for z in 0..dimensions.z {
                for y in 0..=water_level {
                    if map.get_block(x as i16, y as i16, z as i16) == 0x00 {
                        map.set_block(x as i16, y as i16, z as i16, 0x08);
                    }
                }
            }
        }

        for x in 0..dimensions.x {
            for z in 0..dimensions.z {
                for y in (0..dimensions.y).rev() {
                    let block = map.get_block(x as i16, y as i16, z as i16);

                    if block == 0x01 {
                        if map.get_block(x as i16, y as i16 + 1, z as i16) == 0x00
                            || map.get_block(x as i16, y as i16 + 1, z as i16) == 0x08
                        {
                            if y as u32 <= water_level as u32 + 2 {
                                map.set_block(x as i16, y as i16, z as i16, 0x0c);
                            } else {
                                map.set_block(x as i16, y as i16, z as i16, 0x02);
                                for i in 1..=3 {
                                    map.set_block(x as i16, y as i16 - i, z as i16, 0x03);
                                }
                            }
                        }
                    }
                }
            }
        }

        map
    }
}

#[derive(Clone)]
pub struct PresetParams {
    pub flat_ground_level: u32,
    pub noise_layers: Vec<NoiseLayer>,
    pub dimensions: Dimensions,
}

#[derive(Clone)]
pub struct NoiseLayer {
    pub scale: f64,
    pub weight: f64,
}

#[derive(Clone)]
pub struct Dimensions {
    pub x: i16,
    pub y: i16,
    pub z: i16,
}
