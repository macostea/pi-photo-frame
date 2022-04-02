use std::{fs::{self, ReadDir}, io, path::{Path, PathBuf}, rc::Rc};

use rand::prelude::*;

#[derive(Default)]
pub struct PhotoProvider {
    paths: Vec<String>,
    valid_extensions: Vec<String>,
}

impl PhotoProvider {
    pub fn new(paths: Vec<String>) -> Self {
        PhotoProvider {
            paths,
            valid_extensions: vec![
                "jpg".to_string(),
                "jpeg".to_string(),
                "png".to_string(),
                "gif".to_string(),
            ],
        }
    }

    pub fn get_photo(&self) -> Result<PathBuf, io::Error> {
        let mut rng = rand::thread_rng();
        let index = rng.gen_range(0..self.paths.len());

        let mut err: io::Error = io::Error::new(io::ErrorKind::NotFound, "No valid photo found");
        for _ in 0..5 {
            let dir = fs::read_dir(self.paths[index].clone())?;
            let random_photo = PhotoProvider::get_random_entry(dir, self.valid_extensions.clone());
            if random_photo.is_ok() {
                return random_photo;
            }
            
            err = random_photo.unwrap_err();
        }

        return Err(err)
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
