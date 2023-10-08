use std::{
    fs::{self, ReadDir},
    io,
    path::PathBuf,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use exif::{DateTime, In, Tag, Value};
use gtk::{
    gdk_pixbuf::{Pixbuf, PixbufRotation},
    glib::Sender,
};
use rand::prelude::*;
use serde::Deserialize;
use tracing::{debug, instrument, span, warn, Level};

use crate::{geocoder::Geocoder, utils::unsafe_wrapper::UnsafeSendSync};

#[derive(Clone, Debug)]
pub enum Media {
    Photo {
        path: PathBuf,
        orientation: u32,
        location: Option<(f32, f32)>,
        date: Option<String>,
    },
    Video {
        path: PathBuf,
    },
}

pub enum MediaMessage {
    Photo {
        photo: Media,
        photo_data: PhotoData,
        address: Result<String, String>,
    },
    Video {
        video: Media,
    },
}

pub struct PhotoData {
    pub pixbuf: Arc<UnsafeSendSync<Pixbuf>>,
}

#[derive(Deserialize, Default, Debug, Clone)]
pub struct Config {
    pub paths: Vec<String>,
    pub transition_time: u32,
    pub mqtt: bool,
    pub mqtt_host: String,
    pub mqtt_topic: String,
    pub reverse_geocode: bool,
    pub mapbox_api_key: String,
}

#[derive(Default, Debug)]
pub struct MediaProvider {
    config: Config,
    photo_valid_extensions: Vec<String>,
    video_valid_extensions: Vec<String>,
    pub paused: bool,
}

impl MediaProvider {
    pub fn new(config: Config) -> Self {
        MediaProvider {
            config,
            photo_valid_extensions: vec!["jpg".to_string(), "jpeg".to_string(), "png".to_string()],
            video_valid_extensions: vec![
                // "mov".to_string(),
                // "mp4".to_string(),
            ],
            paused: false,
        }
    }

    #[instrument]
    pub fn start_worker(this: Arc<Mutex<MediaProvider>>, media_sender: Sender<MediaMessage>) {
        let config_clone = this.clone().lock().unwrap().config.clone();
        thread::spawn(move || {
            let geocoder = Geocoder::new(config_clone.mapbox_api_key);

            loop {
                let span = span!(Level::TRACE, "get_photo_thread");
                let _enter = span.enter();

                thread::sleep(Duration::from_secs(config_clone.transition_time.into()));
                let media = this.clone().lock().unwrap().get_media();
                debug!("Got media");
                match media {
                    Ok(Some(Media::Photo {
                        ref path,
                        orientation,
                        location,
                        date: _,
                    })) => {
                        let image_data = Pixbuf::from_file(path);
                        if let Err(err) = image_data {
                            warn!("Loading image failed {:?}", err);
                            return;
                        }

                        let pixbuf = Arc::new(UnsafeSendSync::new(image_data.unwrap()));

                        if pixbuf.height() <= 0 || pixbuf.width() <= 0 {
                            warn!("Corrupted image {:?}", path);
                            return;
                        }

                        let new_pixbuf = MediaProvider::rotate_photo(pixbuf, orientation);

                        let mut address_message = Err("Not set".into());
                        if config_clone.reverse_geocode {
                            if let Some(location) = location {
                                debug!("Geolocating");
                                let address = geocoder.reverse_geocode(location.0, location.1);
                                address_message = address;
                                debug!("Finished geolocating");
                            }
                        }

                        let photo_obj = MediaMessage::Photo {
                            photo: media.unwrap().unwrap().clone(),
                            photo_data: PhotoData {
                                pixbuf: new_pixbuf.clone(),
                            },
                            address: address_message,
                        };

                        debug!("Sending photo to UI");
                        let res = media_sender.send(photo_obj);
                        if let Err(e) = res {
                            println!("Failed to send photo_obj between threads {}", e);
                        }
                    }

                    Ok(Some(Media::Video { path: _ })) => {
                        let video_obj = MediaMessage::Video {
                            video: media.unwrap().unwrap().clone(),
                        };

                        let res = media_sender.send(video_obj);
                        if let Err(e) = res {
                            println!("Failed to send video_obj between threads {}", e);
                        }
                    }
                    Ok(None) => {
                        // Everything went ok but there was no media
                        // Most likely paused, don't do anything.
                    }
                    _ => {
                        println!("Error getting photo, {}", media.unwrap_err());
                    }
                }
            }
        });
    }

    #[instrument]
    pub fn get_media(&self) -> Result<Option<Media>, io::Error> {
        if self.paused {
            return Ok(None);
        }

        let mut rng = rand::thread_rng();
        let index = rng.gen_range(0..self.config.paths.len());

        let exifreader = exif::Reader::new();

        for t in 0..5 {
            debug!(current_try = t, "Trying to get a valid photo");
            let dir = fs::read_dir(self.config.paths[index].clone())?;
            let all_extensions = self
                .photo_valid_extensions
                .iter()
                .cloned()
                .chain(self.video_valid_extensions.iter().cloned())
                .collect();

            let random_media_path = MediaProvider::get_random_entry(dir, all_extensions)?;
            let random_media_path_clone = random_media_path.clone();

            let extension = random_media_path.extension().unwrap().to_str().unwrap();

            if self
                .photo_valid_extensions
                .contains(&extension.to_lowercase())
            {
                debug!("Found a valid photo");
                let file = std::fs::File::open(random_media_path)?;
                let mut bufreader = std::io::BufReader::new(file);
                let exif = exifreader
                    .read_from_container(&mut bufreader)
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, e));

                if exif.is_err() {
                    debug!("No exif data");
                    return Ok(Some(Media::Photo {
                        path: random_media_path_clone,
                        orientation: 0,
                        location: None,
                        date: None,
                    }));
                }

                let exif_obj = exif.unwrap();

                let orientation = match exif_obj.get_field(Tag::Orientation, In::PRIMARY) {
                    Some(orientation) => match orientation.value.get_uint(0) {
                        Some(v @ 1..=8) => v,
                        _ => 1,
                    },
                    None => 1,
                };
                debug!(orientation, "Found orientation");

                let latitude = match exif_obj.get_field(Tag::GPSLatitude, In::PRIMARY) {
                    Some(latitude) => match latitude.value {
                        Value::Rational(ref v) if !v.is_empty() => Some(v),
                        _ => None,
                    },
                    None => None,
                };

                let longitude = match exif_obj.get_field(Tag::GPSLongitude, In::PRIMARY) {
                    Some(longitude) => match longitude.value {
                        Value::Rational(ref v) if !v.is_empty() => Some(v),
                        _ => None,
                    },
                    None => None,
                };

                let mut location: Option<(f32, f32)> = None;

                if let Some(lat) = latitude {
                    if let Some(lon) = longitude {
                        let lat_dec: f32 = lat[0].num as f32 / lat[0].denom as f32
                            + (lat[1].num as f32 / lat[1].denom as f32) / 60.0
                            + (lat[2].num as f32 / lat[2].denom as f32) / 3600.0;
                        debug!(lat_dec, "Found latitude");

                        let lon_dec: f32 = lon[0].num as f32 / lon[0].denom as f32
                            + (lon[1].num as f32 / lon[1].denom as f32) / 60.0
                            + (lon[2].num as f32 / lon[2].denom as f32) / 3600.0;
                        debug!(lon_dec, "Found longitude");

                        location = Some((lat_dec, lon_dec));
                    }
                }

                let date_time = match exif_obj.get_field(Tag::DateTime, In::PRIMARY) {
                    Some(date_time) => match date_time.value {
                        Value::Ascii(ref v) if !v.is_empty() => Some(v),
                        _ => None,
                    },
                    None => None,
                };

                let mut string_date_time: Option<String> = None;

                if let Some(ascii_date_time) = date_time {
                    let date_time = DateTime::from_ascii(&ascii_date_time[0]);
                    if date_time.is_ok() {
                        string_date_time = Some(date_time.unwrap().to_string());
                        debug!(string_date_time, "Found time");
                    }
                }

                return Ok(Some(Media::Photo {
                    path: random_media_path_clone,
                    orientation,
                    location,
                    date: string_date_time,
                }));
            } else {
                return Ok(Some(Media::Video {
                    path: random_media_path,
                }));
            }
        }

        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "No valid photo found",
        ));
    }

    #[instrument]
    fn get_random_entry(dir: ReadDir, valid_extensions: Vec<String>) -> Result<PathBuf, io::Error> {
        let mut rng = rand::thread_rng();

        let entry = dir.choose(&mut rng);
        if let Some(entry) = entry {
            let entry = entry?;
            debug!(current_entry = entry.path().to_str(), "Trying an entry");
            if entry.path().is_dir() {
                if let Some(dir) = fs::read_dir(entry.path()).ok() {
                    println!("Dir found, recursing: {:?}", dir);
                    return MediaProvider::get_random_entry(dir, valid_extensions);
                }
            }

            if let Some(extension) = entry.path().extension() {
                if let Some(extension) = extension.to_str() {
                    if valid_extensions.contains(&extension.to_lowercase()) {
                        debug!("Found a valid photo in dir");
                        return Ok(entry.path());
                    }
                }
            }

            println!("Invalid extension: {:?}", entry.path());
        }

        Err(io::Error::new(
            io::ErrorKind::NotFound,
            "No valid photo found",
        ))
    }

    fn rotate_photo(
        pixbuf: Arc<UnsafeSendSync<Pixbuf>>,
        orientation: u32,
    ) -> Arc<UnsafeSendSync<Pixbuf>> {
        // We might need to rotate the image
        debug!("Got pixels");
        let new_pixbuf = match orientation {
            1 => None,
            2 => pixbuf.flip(true),
            3 => pixbuf.rotate_simple(PixbufRotation::Upsidedown),
            4 => pixbuf
                .flip(true)
                .map_or(None, |p| p.rotate_simple(PixbufRotation::Upsidedown)),
            5 => pixbuf
                .flip(true)
                .map_or(None, |p| p.rotate_simple(PixbufRotation::Clockwise)),
            6 => pixbuf.rotate_simple(PixbufRotation::Clockwise),
            7 => pixbuf
                .flip(true)
                .map_or(None, |p| p.rotate_simple(PixbufRotation::Counterclockwise)),
            8 => pixbuf.rotate_simple(PixbufRotation::Counterclockwise),
            _ => None,
        };
        debug!("Flipped pixels");

        new_pixbuf.map_or(pixbuf, |p| Arc::new(UnsafeSendSync::new(p)))
    }
}
