use crate::server::game::dmf_map::DmfMap;
use dashmap::DashMap;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use tokio::io;
use tokio::time::{sleep, Duration};

pub async fn load_all_maps_in(path: &str, maps: Arc<DashMap<String, DmfMap>>) -> io::Result<()> {
    let entries = fs::read_dir(path)?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            let file_name = path.file_stem().unwrap().to_str().unwrap().to_string();
            load_map(&path, maps.clone()).await?;
            println!("map: {} loaded!", file_name);
        }
    }

    Ok(())
}

pub async fn load_map(path: &Path, maps: Arc<DashMap<String, DmfMap>>) -> io::Result<()> {
    let file_name = path.file_stem().unwrap().to_str().unwrap().to_string();
    let map = DmfMap::load_file(path.to_str().unwrap()).unwrap();
    maps.insert(file_name, map);

    Ok(())
}

pub async fn unload_map(name: &str, maps: Arc<DashMap<String, DmfMap>>) {
    maps.remove(name);
    println!("Unloaded map: {}", name);
}

pub async fn save_all(maps: Arc<DashMap<String, DmfMap>>) {
    for r in maps.iter() {
        let (name, map) = r.pair();
        if let Err(e) = map.save_file(&format!("maps/{}.dmf", name)) {
            eprintln!("Failed to save map {}: {}", name, e);
        } else {
            println!("Saved map: {}", name);
        }
    }
}

pub async fn save_all_loop(maps: Arc<DashMap<String, DmfMap>>) {
    loop {
        save_all(Arc::clone(&maps)).await;
        sleep(Duration::from_secs(90)).await;
    }
}
