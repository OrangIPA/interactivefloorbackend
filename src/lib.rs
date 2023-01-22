use std::fs;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub cam_index: i32,
    pub gaussian_blur: GaussianBlur,
    pub kernel_rows: i32,
    pub kernel_cols: i32,
    pub threshold: Threshold,
    pub min_contour_area: f64,
}
impl Config {
    pub fn new() -> Config {
        Config {
            cam_index: 1,
            gaussian_blur: GaussianBlur {
                ksize_width: 5,
                ksize_heigth: 5,
                sigma_x: 0.,
                sigma_y: 0.,
            },
            kernel_rows: 35,
            kernel_cols: 35,
            threshold: Threshold {
                thresh: 20.,
                maxval: 255.,
            },
            min_contour_area: 10000.0,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GaussianBlur {
    pub ksize_width: i32,
    pub ksize_heigth: i32,
    pub sigma_x: f64,
    pub sigma_y: f64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Threshold {
    pub thresh: f64,
    pub maxval: f64,
}

pub const DEFAULT_JSON: &str = r#"{
    "cam_index": 1,
    "gaussian_blur": {
        "ksize_width": 5,
        "ksize_heigth": 5,
        "sigma_x": 0.0,
        "sigma_y": 0.0
    },
    "kernel_rows": 35,
    "kernel_cols": 35,
    "threshold": {
        "thresh": 20.0,
        "maxval": 255.0
    },
    "min_contour_area": 10000.0
}"#;

pub fn json_to_config(v: &str) -> Config {
    let ret = match serde_json::from_str::<Config>(v) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("error: {}, config.json will overridden with default value and continuing with default parameter", e);
            fs::write("config.json", DEFAULT_JSON).unwrap_or_else(|e| {
                eprintln!("{e}");
            });
            Config::new()
        }
    };
    ret
}

#[cfg(test)]
mod test {
    use std::fs;

    use crate::json_to_config;

    #[test]
    fn json_to_config_test() {
        let json = fs::read_to_string("config.json").expect("config.json not found");
        let json = json_to_config(&json);
        assert_eq!(json.cam_index, 1);
    }
}
