use serde::Deserialize;

#[derive(Deserialize)]
struct MapboxResponse {
    features: Vec<MapboxFeatures>,
}

#[derive(Deserialize)]
struct MapboxFeatures {
    place_name: String,
}

#[derive(Default, Debug)]
pub struct Geocoder {
    mapbox_reverse_geocoder_url: String,
}

impl Geocoder {
    pub fn new(mapbox_api_key: String) -> Self {
        Geocoder { mapbox_reverse_geocoder_url: "https://api.mapbox.com/geocoding/v5/mapbox.places/{lon_dec},{lat_dec}.json?types=place&language=ro&access_token={access_token}".replace("{access_token}", &mapbox_api_key) }
    }

    pub fn reverse_geocode(&self, latitude_dec: f32, longitude_dec: f32) -> Result<String, String> {
        let url = self
            .mapbox_reverse_geocoder_url
            .replace("{lat_dec}", latitude_dec.to_string().as_str())
            .replace("{lon_dec}", longitude_dec.to_string().as_str());
        let resp: MapboxResponse = reqwest::blocking::get(url)
            .map_err(|e| e.to_string())?
            .json()
            .map_err(|e| e.to_string())?;

        if !&resp.features.is_empty() {
            let location = &resp.features[0].place_name;
            return Ok(location.into());
        }
        return Err("Reverse geocode response empty".to_string());
    }
}
