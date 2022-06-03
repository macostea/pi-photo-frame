use std::{fs::{self, ReadDir}, io, path::PathBuf};

use exif::{Tag, In, Value, DateTime};
use rand::prelude::*;

#[derive(Clone, Debug)]
pub enum Media {
    Photo {
        path: PathBuf,
        orientation: u32,
        location: Option<(f32, f32)>,
        date: Option<String>,
    },
    Video {
        path: PathBuf
    }
}

#[derive(Default)]
pub struct MediaProvider {
    paths: Vec<String>,
    photo_valid_extensions: Vec<String>,
    video_valid_extensions: Vec<String>,
    pub paused: bool,
}

impl MediaProvider {
    pub fn new(paths: Vec<String>) -> Self {
        MediaProvider {
            paths,
            photo_valid_extensions: vec![
                "jpg".to_string(),
                "jpeg".to_string(),
                "png".to_string(),
            ],
            video_valid_extensions: vec![
                // "mov".to_string(),
                // "mp4".to_string(),
            ],
            paused: false,
        }
    }

    pub fn get_media(&self) -> Result<Media, io::Error> {
        if self.paused {
            return Err(io::Error::new(io::ErrorKind::Interrupted, "Paused"));
        }

        let mut rng = rand::thread_rng();
        let index = rng.gen_range(0..self.paths.len());

        let exifreader = exif::Reader::new();

        for _ in 0..5 {
            let dir = fs::read_dir(self.paths[index].clone())?;
            let all_extensions = self.photo_valid_extensions.iter().cloned().chain(self.video_valid_extensions.iter().cloned()).collect();

            let random_media_path = MediaProvider::get_random_entry(dir, all_extensions)?;
            let random_media_path_clone = random_media_path.clone();

            let extension = random_media_path.extension().unwrap().to_str().unwrap();

            if self.photo_valid_extensions.contains(&extension.to_lowercase()) {
                let file = fs::File::open(random_media_path)?;
                let mut bufreader = std::io::BufReader::new(&file);
                let exif = exifreader.read_from_container(&mut bufreader).map_err(|e| io::Error::new(io::ErrorKind::Other, e));

                if exif.is_err() {
                    return Ok(Media::Photo {
                        path: random_media_path_clone,
                        orientation: 0,
                        location: None,
                        date: None,
                    });
                }

                let exif_obj = exif.unwrap();

                let orientation = match exif_obj.get_field(Tag::Orientation, In::PRIMARY) {
                    Some(orientation) => {
                        match orientation.value.get_uint(0) {
                            Some(v @ 1..=8) => v,
                            _ => 1
                        }
                    },
                    None => 1
                };

                let latitude = match exif_obj.get_field(Tag::GPSLatitude, In::PRIMARY) {
                    Some(latitude) => {
                        match latitude.value {
                            Value::Rational(ref v) if !v.is_empty() => Some(v),
                            _ => None
                        }
                    },
                    None => None
                };

                let longitude = match exif_obj.get_field(Tag::GPSLongitude, In::PRIMARY) {
                    Some(longitude) => {
                        match longitude.value {
                            Value::Rational(ref v) if !v.is_empty() => Some(v),
                            _ => None
                        }
                    },
                    None => None
                };

                let mut location: Option<(f32, f32)> = None;

                if let Some(lat) = latitude {
                    if let Some(lon) = longitude {
                        let lat_dec: f32 = lat[0].num as f32 / lat[0].denom as f32 +
                            (lat[1].num as f32 / lat[1].denom as f32) / 60.0 +
                            (lat[2].num as f32 / lat[2].denom as f32) / 3600.0;

                        let lon_dec: f32 = lon[0].num as f32 / lon[0].denom as f32 +
                            (lon[1].num as f32 / lon[1].denom as f32) / 60.0 +
                            (lon[2].num as f32 / lon[2].denom as f32) / 3600.0;

                        location = Some((lat_dec, lon_dec));
                    }
                }

                let date_time = match exif_obj.get_field(Tag::DateTime, In::PRIMARY) {
                    Some(date_time) => {
                        match date_time.value {
                            Value::Ascii(ref v) if !v.is_empty() => Some(v),
                            _ => None
                        }
                    },
                    None => None
                };

                let mut string_date_time: Option<String> = None;

                if let Some(ascii_date_time) = date_time {
                    let date_time = DateTime::from_ascii(&ascii_date_time[0]);
                    if date_time.is_ok() {
                        string_date_time = Some(date_time.unwrap().to_string());
                    }
                }

                return Ok(Media::Photo {
                    path: random_media_path_clone,
                    orientation,
                    location,
                    date: string_date_time,
                });
            } else {
                return Ok(Media::Video { path: random_media_path });
            }
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
                    return MediaProvider::get_random_entry(dir, valid_extensions);
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
