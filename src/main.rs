use metrohash::MetroHash;
use std::fs::metadata;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use structopt::clap::arg_enum;
use structopt::StructOpt;

mod bin_packs;
mod error;
mod image_wrapper;
mod packer;
mod path_glob;
mod rect;
mod serial;

use error::Result;
use image_wrapper::ImageWrapper;
use path_glob::Glob;

// Trait for extending std::path::PathBuf
use path_slash::PathBufExt;

arg_enum! {
    #[derive(Debug, Copy, Clone, Hash)]
    enum FreeRectChoiceHeuristic {
        BestShortSideFit,
        BestLongSideFit,
        BestAreaFit,
        BottomLeftRule,
        ContactPointRule,
    }
}

impl Into<bin_packs::max_rects::FreeRectChoiceHeuristic> for FreeRectChoiceHeuristic {
    fn into(self) -> bin_packs::max_rects::FreeRectChoiceHeuristic {
        match self {
            FreeRectChoiceHeuristic::BestShortSideFit => {
                bin_packs::max_rects::FreeRectChoiceHeuristic::RectBestShortSideFit
            }
            FreeRectChoiceHeuristic::BestLongSideFit => {
                bin_packs::max_rects::FreeRectChoiceHeuristic::RectBestLongSideFit
            }
            FreeRectChoiceHeuristic::BestAreaFit => {
                bin_packs::max_rects::FreeRectChoiceHeuristic::RectBestAreaFit
            }
            FreeRectChoiceHeuristic::BottomLeftRule => {
                bin_packs::max_rects::FreeRectChoiceHeuristic::RectBottomLeftRule
            }
            FreeRectChoiceHeuristic::ContactPointRule => {
                bin_packs::max_rects::FreeRectChoiceHeuristic::RectContactPointRule
            }
        }
    }
}

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
    #[structopt(short, long, parse(from_occurrences))]
    verbose: u8,

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
    #[structopt(short = "P", long, default_value = "1")]
    pad: u8,

    /// The image-packing heuristic to use
    #[structopt(short, long, possible_values = &FreeRectChoiceHeuristic::variants(), default_value = "BestShortSideFit", case_insensitive = true)]
    heuristic: FreeRectChoiceHeuristic,

    /// The image format to use when saving atlas images
    #[structopt(short, long, default_value = "png", possible_values = &["ico", "jpg", "jpeg", "png", "pbm", "pgm", "ppm", "pam", "bmp", "tif", "tiff"], case_insensitive = true)]
    extension: String,

    /// File to output
    #[structopt(name = "OUTPUT", parse(from_os_str))]
    output: PathBuf,

    /// Files or folders to process
    #[structopt(name = "INPUTS", parse(from_os_str))]
    inputs: Vec<PathBuf>,
}

/// Use the available extensions in the `image` crate to determine if a file extension
/// is associated with an image or not.
fn is_image_file<P: AsRef<std::path::Path>>(path: P) -> bool {
    let p = path.as_ref();
    let ext = p
        .extension()
        .and_then(|s| s.to_str())
        .map_or("".to_string(), |s| s.to_ascii_lowercase());
    match &*ext {
        "ico" => true,
        "jpg" | "jpeg" => true,
        "png" => true,
        "pbm" => true,
        "pgm" => true,
        "ppm" => true,
        "pam" => true,
        "bmp" => true,
        "tif" | "tiff" => true,
        _ => false,
    }
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
    if is_image_file(path) {
        let bytes = std::fs::read(path)?;
        hasher.write(&bytes);
    }
    Ok(())
}

fn load_image<P: AsRef<std::path::Path>>(
    path: P,
    images: &mut Vec<ImageWrapper>,
    opt: &Opt,
) -> Result<()> {
    if is_image_file(&path) {
        log::info!("Reading file {}", path.as_ref().to_string_lossy());
        let size = std::fs::metadata(path.as_ref())?.len();
        let img = image::open(path.as_ref().clone())?.to_rgba();
        let mut given_path = path.as_ref().to_path_buf();
        given_path.pop();
        given_path.push(path.as_ref().file_stem().unwrap());
        let img = ImageWrapper::new(
            img,
            given_path.to_slash().unwrap(),
            opt.premultiply,
            opt.trim,
            size,
        );
        images.push(img);
    } else {
        log::info!(
            "File {} is not an image, skipping...",
            path.as_ref().to_string_lossy()
        );
    }
    Ok(())
}

fn load_images<P: AsRef<std::path::Path>>(
    path: P,
    images: &mut Vec<ImageWrapper>,
    opt: &Opt,
) -> Result<()> {
    let dir_iter = std::fs::read_dir(&path)?;
    log::info!("Reading directory {}", path.as_ref().to_string_lossy());
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

    let log_level = match opt.verbose {
        0 => log::LevelFilter::Warn,
        1 => log::LevelFilter::Info,
        2 => log::LevelFilter::Debug,
        _ => log::LevelFilter::Trace,
    };

    let file_config = fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.target(),
                record.level(),
                message
            ))
        })
        .chain(fern::log_file("impact.log")?)
        .level(log::LevelFilter::Trace);

    let stderr_config = fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.target(),
                record.level(),
                message
            ))
        })
        .level(log_level)
        .chain(std::io::stderr());
    
    fern::Dispatch::new()
        .chain(file_config)
        .chain(stderr_config)
        .apply()?;

    if opt.pad > 16 {
        log::error!("Invalid padding value: {}", opt.pad);
        return Err(error::ImpactError::InvalidPadding { size: opt.pad });
    }

    let output_dir = opt
        .output
        .parent()
        .expect("could not retrieve output directory");
    let output_name = opt
        .output
        .file_name()
        .expect("could not retrieve output filename");

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
    let hash_path = output_dir
        .join(&format!("{}", output_name.to_string_lossy()))
        .with_extension("hash");
    if hash_path.exists() {
        let contents = std::fs::read_to_string(&hash_path)?;
        if !opt.force && contents == hash_str {
            log::info!("Atlas is unchanged: {}", output_name.to_string_lossy());
            return Ok(());
        }
    }

    log::trace!("Options:\n{:?}", opt);

    // Remove old files
    if hash_path.exists() {
        std::fs::remove_file(&hash_path)?;
    }
    let bin_path = output_dir
        .join(&format!("{}", output_name.to_string_lossy()))
        .with_extension("bin");
    if bin_path.exists() {
        std::fs::remove_file(&bin_path)?;
    }

    let xml_path = output_dir
        .join(&format!("{}", output_name.to_string_lossy()))
        .with_extension("xml");
    if xml_path.exists() {
        std::fs::remove_file(&xml_path)?;
    }

    let json_path = output_dir
        .join(&format!("{}", output_name.to_string_lossy()))
        .with_extension("json");
    if json_path.exists() {
        std::fs::remove_file(&json_path)?;
    }

    for atlas in output_dir
        .glob(&format!(
            "{}*.{}",
            output_name.to_string_lossy(),
            &opt.extension
        ))
        .expect("failed to read glob pattern")
    {
        match atlas {
            Ok(path) => std::fs::remove_file(&path)?,
            Err(_) => (),
        }
    }

    // Load the bitmaps from all the input files and directories
    log::info!("loading images...");
    let mut images = vec![];
    for input in &opt.inputs {
        let md = metadata(input)?;
        if md.is_dir() {
            load_images(input, &mut images, &opt)?;
        } else {
            load_image(input, &mut images, &opt)?;
        }
    }
    log::info!("loaded {} images.", images.len());
    
    {
        use humansize::{FileSize, file_size_opts as options};
        let size = images.iter().fold(0, |sum, img| sum + img.original_size);
        log::info!("size of all images: {}", size.file_size(options::CONVENTIONAL).unwrap());
    }

    // Sort the bitmaps by area
    images.sort_unstable_by(|a: &ImageWrapper, b: &ImageWrapper| {
        (a.width * a.height).cmp(&(b.width * b.height))
    });

    // Pack the bitmaps
    let mut packers = vec![];
    while !images.is_empty() {
        log::info!("packing {} images...", images.len());
        let mut packer = packer::Packer::new(opt.size as i32, opt.size as i32, opt.pad as i32);
        packer.pack(
            &mut images,
            opt.unique,
            opt.rotate,
            opt.heuristic.into(),
        );
        log::info!(
                "finished packing {} - ({}x{})",
                packers.len(),
                packer.width,
                packer.height
            );
        if packer.images.is_empty() {
            log::error!(
                "packing failed, could not fit image {}",
                images.first().unwrap().name
            );
            return Err(error::ImpactError::CantFitError);
        }
        packers.push(packer);
    }

    // Save the atlas image
    for (idx, packer) in packers.iter().enumerate() {
        let out_path = output_dir
            .join(&format!("{}{}", output_name.to_string_lossy(), idx))
            .with_extension(&opt.extension);
        log::info!("writing image {}", out_path.display());
        packer.save_png(out_path)?;
    }

    // Create info
    let mut atlas = serial::Atlas { textures: vec![] };

    for (idx, packer) in packers.iter().enumerate() {
        let name = output_name.to_string_lossy();
        let mut texture = serial::Texture {
            name: format!("{}{}", name, idx),
            images: vec![],
        };
        for (img_idx, img) in packer.images.iter().enumerate() {
            let p = &packer.points[img_idx];
            let s_img = serial::Image {
                name: String::from(&img.name),
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
        let out_path = output_dir
            .join(&format!("{}", output_name.to_string_lossy()))
            .with_extension("bin");
        log::info!("writing binary {}", out_path.display());
        let res = bincode::serialize(&atlas).expect("failed to serialize into binary data");
        std::fs::write(out_path, &res)?;
    }

    // Save the atlas xml
    if opt.xml {
        let out_path = output_dir
            .join(&format!("{}", output_name.to_string_lossy()))
            .with_extension("xml");
        log::info!("writing xml {}", out_path.display());
        atlas.write_to_xml_file(out_path)?;
    }

    // Save the atlas json
    if opt.json {
        let out_path = output_dir
            .join(&format!("{}", output_name.to_string_lossy()))
            .with_extension("json");
        log::info!("writing json {}", out_path.display());
        let res = serde_json::to_vec_pretty(&atlas).expect("failed to serialize into json");
        std::fs::write(out_path, &res)?;
    }

    // Save the new hash
    std::fs::write(&hash_path, hash_str)?;
    Ok(())
}
