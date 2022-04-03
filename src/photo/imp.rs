use std::{fs::{self, ReadDir}, io, path::PathBuf};

use exif::{Tag, In};
use rand::prelude::*;

#[derive(Clone, Debug)]
pub struct Photo {
    pub path: PathBuf,
    pub orientation: u32,
}

#[derive(Default)]
pub struct PhotoProvider {
    paths: Vec<String>,
    valid_extensions: Vec<String>,
    pub paused: bool,
}

impl PhotoProvider {
    pub fn new(paths: Vec<String>) -> Self {
        PhotoProvider {
            paths,
            valid_extensions: vec![
                "jpg".to_string(),
                "jpeg".to_string(),
                "png".to_string(),
            ],
            paused: false,
        }
    }

    pub fn get_photo(&self) -> Result<Photo, io::Error> {
        if self.paused {
            return Err(io::Error::new(io::ErrorKind::Interrupted, "Paused"));
        }

        let mut rng = rand::thread_rng();
        let index = rng.gen_range(0..self.paths.len());

        let exifreader = exif::Reader::new();

        for _ in 0..5 {
            let dir = fs::read_dir(self.paths[index].clone())?;
            let random_photo_path = PhotoProvider::get_random_entry(dir, self.valid_extensions.clone())?;
            let random_photo_path_clone = random_photo_path.clone();

            let file = fs::File::open(random_photo_path)?;
            let mut bufreader = std::io::BufReader::new(&file);
            let exif = exifreader.read_from_container(&mut bufreader).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

            let orientation = match exif.get_field(Tag::Orientation, In::PRIMARY) {
                Some(orientation) => {
                    match orientation.value.get_uint(0) {
                        Some(v @ 1..=8) => v,
                        _ => 1
                    }
                },
                None => 1
            };

            return Ok(Photo {
                path: random_photo_path_clone,
                orientation
            });
        }

        return Err(io::Error::new(io::ErrorKind::NotFound, "No valid photo found"))
    }

    fn get_random_entry(dir: ReadDir, valid_extensions: Vec<String>) -> Result<PathBuf, io::Error> {
        let mut rng = rand::thread_rng();

        let entry = dir.choose(&mut rng);
        if let Some(entry) = entry {
            let entry = entry?;
            if entry.path().is_dir() {
                if let Some(dir) = fs::read_dir(entry.path()).ok() {
                    println!("Dir found, recursing: {:?}", dir);
                    return PhotoProvider::get_random_entry(dir, valid_extensions);
                }
            }

            if let Some(extension) = entry.path().extension() {
                if let Some(extension) = extension.to_str() {
                    if valid_extensions.contains(&extension.to_lowercase()) {
                        return Ok(entry.path());
                    }
                }
            }

            println!("Invalid extension: {:?}", entry.path());
        }

        Err(io::Error::new(io::ErrorKind::NotFound, "No valid photo found"))
    }
}
