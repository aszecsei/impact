use std::path::PathBuf;
use structopt::StructOpt;
use metrohash::{MetroHash};
use std::hash::{Hash, Hasher};
use std::fs::metadata;

mod bin_packs;
mod error;
mod image_wrapper;
mod packer;
mod path_glob;
mod rect;
mod serial;

use error::Result;
use path_glob::Glob;
use image_wrapper::ImageWrapper;

/// A texture packer
#[derive(StructOpt, Debug, Hash)]
#[structopt(name = "impact")]
struct Opt {
    /// Use default settings (-x -p -t -u)
    #[structopt(short, long)]
    default: bool,

    /// Saves the atlas data as a .xml file
    #[structopt(short, long)]
    xml: bool,

    /// Saves the atlas data as a .bin file
    #[structopt(short, long)]
    binary: bool,

    /// Saves the atlas data as a .json file
    #[structopt(short, long)]
    json: bool,

    /// Premultiplies the pixels of the bitmaps by their alpha channel
    #[structopt(short, long)]
    premultiply: bool,
    
    /// Trims excess transparency off the bitmaps
    #[structopt(short, long)]
    trim: bool,

    /// Print to the debug console as the packer works
    #[structopt(short, long)]
    verbose: bool,

    /// Ignore caching, forcing the packer to repack
    #[structopt(short, long)]
    force: bool,

    /// Remove duplicate bitmaps from the atlas
    #[structopt(short, long)]
    unique: bool,

    /// Enables rotating bitmaps 90 degrees clockwise when packing
    #[structopt(short, long)]
    rotate: bool,

    /// Max atlas size
    #[structopt(short, long, default_value = "4096", possible_values = &["64", "128", "256", "512", "1024", "2048", "4096"])]
    size: u16,

    /// Padding between images (can be from 0 to 16)
    #[structopt(long, default_value = "1")]
    pad: u8,

    /// File to output
    #[structopt(name = "OUTPUT", parse(from_os_str))]
    output: PathBuf,

    /// Files or folders to process
    #[structopt(name = "INPUTS", parse(from_os_str))]
    inputs: Vec<PathBuf>,
}

fn hash_files(path: &PathBuf, hasher: &mut dyn std::hash::Hasher) -> Result<()> {
    let dir_iter = std::fs::read_dir(path)?;
    for dir in dir_iter {
        let dir = dir?;
        if dir.metadata()?.is_dir() {
            hash_files(&dir.path(), hasher)?;
        } else {
            hash_file(&dir.path(), hasher)?;
        }
    }
    Ok(())
}

fn hash_file(path: &PathBuf, hasher: &mut dyn std::hash::Hasher) -> Result<()> {
    let bytes = std::fs::read(path)?;
    hasher.write(&bytes);
    Ok(())
}

fn load_image<P: AsRef<std::path::Path>>(path: P, images: &mut Vec<ImageWrapper>, opt: &Opt) -> Result<()> {
    let img = image::open(path.as_ref().clone())?.to_rgba();
    let img = ImageWrapper::new(img, String::from(path.as_ref().to_str().unwrap()), opt.premultiply, opt.trim);
    images.push(img);
    Ok(())
}

fn load_images<P: AsRef<std::path::Path>>(path: P, images: &mut Vec<ImageWrapper>, opt: &Opt) -> Result<()> {
    let dir_iter = std::fs::read_dir(path)?;
    for dir in dir_iter {
        let dir = dir?;
        if dir.metadata()?.is_dir() {
            load_images(&dir.path(), images, opt)?;
        } else {
            load_image(&dir.path(), images, opt)?;
        }
    }
    Ok(())
}

fn main() -> Result<()> {
    let mut opt = Opt::from_args();

    if opt.default {
        opt.xml = true;
        opt.premultiply = true;
        opt.trim = true;
        opt.unique = true;
    }

    if opt.pad > 16 {
        eprintln!("Invalid padding value: {}", opt.pad);
        return Err(error::ImpactError::InvalidPadding { size: opt.pad });
    }

    let output_dir = opt.output.parent().expect("could not retrieve output directory");
    let output_name = opt.output.file_name().expect("could not retrieve output filename");

    // Hash the arguments and input directories
    let mut hasher = MetroHash::default();
    opt.hash(&mut hasher);
    for input in &opt.inputs {
        let md = metadata(input)?;
        if md.is_dir() {
            hash_files(input, &mut hasher)?;
        } else {
            hash_file(input, &mut hasher)?;
        }
    }
    let hash = hasher.finish();
    let hash_str = format!("{}", hash);

    // Load the old hash
    let hash_path = {
        let mut hash_path = output_dir.clone().to_path_buf();
        hash_path.push(output_name);
        hash_path.set_extension("hash");
        hash_path
    };
    if hash_path.exists() {
        let contents = std::fs::read_to_string(&hash_path)?;
        if !opt.force && contents == hash_str {
            println!("Atlas is unchanged: {}", output_name.to_string_lossy());
            return Ok(())
        }
    }

    if opt.verbose {
        println!("Options:\n{:?}", opt);
    }

    // Remove old files
    let mut hash_path = output_dir.clone().to_path_buf();
    hash_path.push(output_name);
    hash_path.set_extension("hash");
    if hash_path.exists() {
        std::fs::remove_file(&hash_path)?;
    }
    hash_path.set_extension("bin");
    if hash_path.exists() {
        std::fs::remove_file(&hash_path)?;
    }
    hash_path.set_extension("xml");
    if hash_path.exists() {
        std::fs::remove_file(&hash_path)?;
    }
    hash_path.set_extension("json");
    if hash_path.exists() {
        std::fs::remove_file(&hash_path)?;
    }
    hash_path.pop();
    for atlas in hash_path.glob(&format!("{}*.png", output_name.to_string_lossy())).expect("failed to read glob pattern") {
        match atlas {
            Ok(path) => std::fs::remove_file(&path)?,
            Err(_) => ()
        }
    }

    // Load the bitmaps from all the input files and directories
    if opt.verbose {
        println!("loading images...");
    }
    let mut images = vec![];
    for input in &opt.inputs {
        let md = metadata(input)?;
        if md.is_dir() {
            load_images(input, &mut images, &opt)?;
        } else {
            load_image(input, &mut images, &opt)?;
        }
    }

    // Sort the bitmaps by area
    images.sort_unstable_by(|a: &ImageWrapper, b: &ImageWrapper| {
        (a.width * a.height).cmp(&(b.width * b.height))
    });

    // Pack the bitmaps
    let mut packers = vec![];
    while !images.is_empty() {
        if opt.verbose {
            println!("packing {} images...", images.len());
        }
        let mut packer = packer::Packer::new(opt.size as i32, opt.size as i32, opt.pad as i32);
        packer.pack(&mut images, opt.verbose, opt.unique, opt.rotate);
        
        if opt.verbose {
            println!("finished packing {} - ({}x{})", packers.len(), packer.width, packer.height);
        }
        if packer.images.is_empty() {
            eprintln!("packing failed, could not fit image {}", images.first().unwrap().name);
            return Err(error::ImpactError::CantFitError);
        }
        packers.push(packer);
    }

    // Save the atlas image
    for (idx, packer) in packers.iter().enumerate() {
        let out_path = output_dir.join(&format!("{}{}", output_name.to_string_lossy(), idx)).with_extension(".png");
        if opt.verbose {
            println!("writing png {}", out_path.display());
        }
        packer.save_png(output_dir.join(output_name).with_extension(".png"))?;
    }

    // Create info
    let mut atlas = serial::Atlas {
        textures: vec![],
    };

    for (idx, packer) in packers.iter().enumerate() {
        let name = output_name.to_string_lossy();
        let mut texture = serial::Texture {
            name: format!("{}{}", name, idx),
            images: vec![],
        };
        for (img_idx, img) in packer.images.iter().enumerate() {
            let p = &packer.points[img_idx];
            let s_img = serial::Image {
                x: p.x,
                y: p.y,
                width: img.width,
                height: img.height,
                frame_x: img.frame_x,
                frame_y: img.frame_y,
                frame_width: img.frame_w,
                frame_height: img.frame_h,
                rotated: p.rot,
            };
            texture.images.push(s_img);
        }
        atlas.textures.push(texture);
    }

    // Save the atlas binary
    if opt.binary {
        let out_path = output_dir.join(&format!("{}", output_name.to_string_lossy())).with_extension(".bin");
        if opt.verbose {
            println!("writing binary {}", out_path.display());
        }
        let res = bincode::serialize(&atlas).expect("failed to serialize into binary data");
        std::fs::write(out_path, &res)?;
    }

    // Save the atlas xml
    if opt.xml {
        let out_path = output_dir.join(&format!("{}", output_name.to_string_lossy())).with_extension(".xml");
        if opt.verbose {
            println!("writing xml {}", out_path.display());
        }
        let res = serde_xml_rs::to_string(&atlas).expect("failed to serialize into xml");
        std::fs::write(out_path, &res)?;
    }

    // Save the atlas json
    if opt.json {
        let out_path = output_dir.join(&format!("{}", output_name.to_string_lossy())).with_extension(".json");
        if opt.verbose {
            println!("writing json {}", out_path.display());
        }
        let res = serde_json::to_vec(&atlas).expect("failed to serialize into json");
        std::fs::write(out_path, &res)?;
    }

    // Save the new hash
    std::fs::write(&hash_path, hash_str)?;
    
    Ok(())
}
