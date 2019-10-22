use image::DynamicImage;
use metrohash::MetroHashMap;

pub struct Point {
    pub x: i32,
    pub y: i32,
    pub dupID: i32,
    pub rot: bool,
}

pub struct Packer {
    pub width: i32,
    pub height: i32,
    pub pad: i32,

    pub images: Vec<DynamicImage>,
    pub points: Vec<Point>,
    pub dup_lookup: MetroHashMap<usize, i32>,
}

impl Packer {
    pub fn new(width: i32, height: i32, pad: i32) -> Self {
        Self {
            width,
            height,
            pad,

            images: vec![],
            points: vec![],
            dup_lookup: MetroHashMap::default(),
        }
    }

    pub fn pack(images: &[DynamicImage], verbose: bool, unique: bool, rotate: bool) {

    }

    pub fn save_png<P: AsRef<std::path::Path>>(file: P) {

    }

    pub fn save_xml<P: AsRef<std::path::Path>>(file: P) {

    }

    pub fn save_bin<P: AsRef<std::path::Path>>(file: P) {

    }

    pub fn save_json<P: AsRef<std::path::Path>>(file: P) {

    }
}