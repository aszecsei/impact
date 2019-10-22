use std::path::PathBuf;
use structopt::StructOpt;
use metrohash::{MetroHash};
use std::hash::{Hash, Hasher};
use std::fs::{File, metadata};
use std::io::BufReader;
use std::io::prelude::*;

mod bin_packs;
mod error;
mod packer;
mod path_glob;
mod rect;

use error::Result;
use path_glob::Glob;

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

fn hash_files(path: &PathBuf, hasher: &mut std::hash::Hasher) -> Result<()> {
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

fn hash_file(path: &PathBuf, hasher: &mut std::hash::Hasher) -> Result<()> {
    let bytes = std::fs::read(path)?;
    hasher.write(&bytes);
    Ok(())
}

fn load_image<P: AsRef<std::path::Path>>(path: P, images: &mut Vec<image::DynamicImage>) -> Result<()> {
    let img = image::open(path)?;
    images.push(img);
    Ok(())
}

fn load_images<P: AsRef<std::path::Path>>(path: P, images: &mut Vec<image::DynamicImage>) -> Result<()> {
    let dir_iter = std::fs::read_dir(path)?;
    for dir in dir_iter {
        let dir = dir?;
        if dir.metadata()?.is_dir() {
            load_images(&dir.path(), images)?;
        } else {
            load_image(&dir.path(), images)?;
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
    let images = vec![];
    for input in &opt.inputs {
        let md = metadata(input)?;
        if md.is_dir() {
            load_images(input, &mut images)?;
        } else {
            load_image(input, &mut images)?;
        }
    }

    // Sort the bitmaps by area

    // Pack the bitmaps

    // Save the atlas image

    // Save the atlas binary

    // Save the atlas xml

    // Save the atlas json

    // Save the new hash
    std::fs::write(&hash_path, hash_str)?;
    
    Ok(())
}
