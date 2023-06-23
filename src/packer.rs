use crate::bin_packs::max_rects::{FreeRectChoiceHeuristic, MaxRectsBinPack};
use crate::error::Result;
use crate::image_wrapper::ImageWrapper;
use metrohash::MetroHashMap;

#[derive(Debug, Clone)]
pub struct Point {
    pub x: i32,
    pub y: i32,
    pub dup_id: i32,
    pub rot: bool,
}

pub struct Packer {
    pub width: i32,
    pub height: i32,
    pub pad: i32,

    pub images: Vec<ImageWrapper>,
    pub points: Vec<Point>,
    pub dup_lookup: MetroHashMap<u64, usize>,
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

    pub fn pack(
        &mut self,
        images: &mut Vec<ImageWrapper>,
        unique: bool,
        rotate: bool,
        method: FreeRectChoiceHeuristic,
    ) {
        let mut packer = MaxRectsBinPack::new(self.width, self.height);

        let mut ww = 0;
        let mut hh = 0;

        log::info!("packing begin...");

        while !images.is_empty() {
            let image = images.pop().unwrap();

            log::info!("{}: {}", images.len(), image.name);

            if unique {
                if self.dup_lookup.contains_key(&image.hash_value) {
                    let idx = self.dup_lookup[&image.hash_value];
                    if image == self.images[idx] {
                        let mut p = self.points[idx].clone();
                        p.dup_id = idx as i32;
                        self.points.push(p);
                        self.images.push(image);

                        log::info!("duplicate found");

                        continue;
                    }
                }
            }

            // If it's not a duplicate, pack it into the atlas
            {
                let rect = packer.insert(
                    image.width + self.pad,
                    image.height + self.pad,
                    rotate,
                    method,
                );

                if rect.width == 0 || rect.height == 0 {
                    images.push(image);
                    break;
                }

                if unique {
                    self.dup_lookup.insert(image.hash_value, self.points.len());
                }

                // Check if we rotated it
                let p = Point {
                    x: rect.x,
                    y: rect.y,
                    dup_id: -1,
                    rot: rotate && image.width != (rect.width - self.pad),
                };

                self.points.push(p);
                self.images.push(image);

                ww = std::cmp::max(rect.x + rect.width, ww);
                hh = std::cmp::max(rect.y + rect.height, hh);
            }
        }

        log::info!("packing complete. resizing...");

        while self.width / 2 >= ww {
            self.width /= 2;
        }
        while self.height / 2 >= hh {
            self.height /= 2;
        }
    }

    pub fn save_png<P: AsRef<std::path::Path>>(&self, file: P) -> Result<()> {
        let mut img = ImageWrapper::empty(self.width, self.height);
        for i in 0..self.images.len() {
            if self.points[i].dup_id < 0 {
                if self.points[i].rot {
                    img.copy_pixels_rot(&self.images[i], self.points[i].x, self.points[i].y);
                } else {
                    img.copy_pixels(&self.images[i], self.points[i].x, self.points[i].y);
                }
            }
        }
        img.save_as(file.as_ref())?;

        {
            use humansize::{format_size, DECIMAL};
            let size = std::fs::metadata(file.as_ref())?.len();
            log::info!("saving atlas. image size: {}", format_size(size, DECIMAL));
        }

        Ok(())
    }
}
