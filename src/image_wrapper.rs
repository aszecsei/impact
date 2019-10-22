use image::{Pixel, RgbaImage};
use metrohash::{MetroHash};
use std::hash::{Hash, Hasher};
use crate::error::Result;

pub struct ImageWrapper {
    pub name: String,
    pub width: i32,
    pub height: i32,
    pub frame_x: i32,
    pub frame_y: i32,
    pub frame_w: i32,
    pub frame_h: i32,
    pub data: Vec<u8>,
    pub hash_value: u64,
}

impl ImageWrapper {
    pub fn new(mut image: RgbaImage, name: String, premultiply: bool, trim: bool) -> Self {
        let w = image.width() as i32;
        let h = image.height() as i32;
        let mut pixels = image.into_vec();

        // premultiply all pixels by their alpha
        if premultiply {
            let count = (w as usize) * (h as usize);
            for i in 0..count {
                let r = pixels[i * 4 + 0];
                let g = pixels[i * 4 + 1];
                let b = pixels[i * 4 + 2];
                let a = pixels[i * 4 + 3] as f32 / 255f32;

                pixels[i * 4 + 0] = (r as f32 * a) as u8;
                pixels[i * 4 + 1] = (g as f32 * a) as u8;
                pixels[i * 4 + 2] = (b as f32 * a) as u8;
            }
        }

        // get pixel bounds
        let mut min_x = w - 1;
        let mut min_y = h - 1;
        let mut max_x = 0;
        let mut max_y = 0;
        if trim {

        } else {
            min_x = 0;
            min_y = 0;
            max_x = w - 1;
            max_y = h - 1;
        }

        // calculate our trimmed size
        let width = (max_x - min_x) + 1;
        let height = (max_x - min_y) + 1;
        let frame_w = w;
        let frame_h = h;

        let (frame_x, frame_y, data) = if width == w {
            (0, 0, pixels)
        } else {
            // create the trimmed image data
            let mut data = Vec::<u8>::with_capacity((width * height) as usize);
            let frame_x = -min_x;
            let frame_y = -min_y;

            // copy trimmed pixels over to the trimmed pixel array
            for y in min_y..max_y+1 {
                for x in min_x..max_x+1 {
                    data[((y - min_y) * width + (x - min_x)) as usize] = pixels[(y * w + x) as usize];
                }
            }

            (frame_x, frame_y, data)
        };

        // generate a hash for the bitmap
        let mut hash = MetroHash::default();
        hash.write_i32(width);
        hash.write_i32(height);
        for byte in data.iter() {
            hash.write_u8(byte.clone());
        }
        let hash_value = hash.finish();

        Self {
            name,
            width,
            height,
            frame_x,
            frame_y,
            frame_w,
            frame_h,
            data,
            hash_value,
        }
    }

    pub fn empty(width: i32, height: i32) -> Self {
        Self {
            name: String::new(),
            width,
            height,
            frame_x: 0,
            frame_y: 0,
            frame_w: width,
            frame_h: width,
            data: vec![],
            hash_value: 0,
        }
    }

    pub fn get_image(&self) -> RgbaImage {
        RgbaImage::from_vec(self.width as u32, self.height as u32, self.data.clone()).unwrap()
    }

    pub fn save_as<P: AsRef<std::path::Path>>(&self, name: P) -> Result<()> {
        let img = self.get_image();
        img.save(name)?;
        Ok(())
    }

    pub fn copy_pixels(&mut self, src: &ImageWrapper, tx: i32, ty: i32) {
        for y in 0..src.height {
            for x in 0..src.width {
                self.data[((ty + y) * self.width + (tx + x)) as usize] = src.data[(y * src.width + x) as usize];
            }
        }
    }

    pub fn copy_pixels_rot(&mut self, src: &ImageWrapper, tx: i32, ty: i32) {
        let r = src.height - 1;
        for y in 0..src.width {
            for x in 0..src.height {
                self.data[((ty + y) * self.width + (tx + x)) as usize] = src.data[((r - x) * src.width + y) as usize];
            }
        }
    }
}

impl PartialEq for ImageWrapper {
    fn eq(&self, other: &Self) -> bool {
        if self.width == other.width && self.height == other.height {
            return self.data == other.data;
        }
        false
    }
}

impl Eq for ImageWrapper {}