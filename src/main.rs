use anyhow::{ensure, Context, Result};
use clap::{Parser, ValueEnum};
use directories::UserDirs;
use image::{imageops::FilterType, DynamicImage, GenericImage, Rgba, RgbaImage};
use log::info;
use std::path::PathBuf;

#[allow(dead_code)]
#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Orientation {
    Portrait,
    Landscape,
}

#[derive(Parser, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[command(name = "Collage")]
#[command(author = "Jeffrey M. Rosenbluth")]
#[command(version = "0.1")]
#[command(about = "Create a collage from a iist of images", long_about = None)]
/// Create a collage from a list of images.
///
/// Collage can either be a column (portrait) or a row (landscape) of images.
/// User can select and orientation, background color, margins and spacing.
/// All images are resized to the same size specified by the user. Size will
/// default to the size of the first image.
struct App {
    /// The paths to the images to be used in the collage.
    image_paths: Vec<PathBuf>,

    /// The width of the images in the collage. If not specified, the width of
    /// the first image will be used.
    #[arg(long = "width", short = 'w')]
    image_width: Option<u32>,

    /// The height of the images in the collage. If not specified, the height of
    /// the first image will be used.
    #[arg(long = "height", short = 'h')]
    image_height: Option<u32>,

    /// The orientation of the collage. If not specified, the default is
    /// `portrait`.
    #[arg(long, short, default_value = "portrait")]
    orientation: Orientation,

    /// The top and bottom margin of the collage. If not specified, the default is 0.
    #[arg(long = "top", short = 't', default_value_t = 0)]
    top_margin: u32,

    /// The left and right margin of the collage. If not specified, the default is 0.
    #[arg(long = "left", short = 'l', default_value_t = 0)]
    left_margin: u32,

    /// The spacing between images. If not specified, the default is 20.
    #[arg(long = "spacing", short = 's', default_value_t = 20)]
    spacing: u32,

    /// The background color of the collage. If not specified, the default is
    /// white.
    #[arg(long = "color", short = 'c', default_value = "#ffffff")]
    background_color: String,

    /// If true, then the aspect ratio of the images will be preserved. If not
    /// specified, the default is false.
    #[arg(long = "preserve", short = 'p', default_value_t = false)]
    preserve_aspect_ratio: bool,
}

#[derive(Debug, Clone)]
struct Model {
    images: Vec<DynamicImage>,
    image_width: u32,
    image_height: u32,
}

// Resize an image to the specified width and height. If preserve_aspect_ratio
// is true, then the image will be resized so that if `Portrait` orientation
// then the width will be set to width and the heigth to width / aspect ration.
// If it's `Landscape` then the width will be set to height * aspect ratio.
fn prepare_image(image: &DynamicImage, width: u32, height: u32, app: &App) -> DynamicImage {
    // If we're not preserving the aspect ratio, just resize to the exact width and height.
    if !app.preserve_aspect_ratio {
        return image.resize_exact(width, height, FilterType::CatmullRom);
    };

    let aspect_ratio = image.width() as f32 / image.height() as f32;

    let (w, h) = match app.orientation {
        Orientation::Landscape => ((height as f32 * aspect_ratio) as u32, height),
        Orientation::Portrait => (width, (width as f32 / aspect_ratio) as u32),
    };

    image.resize_exact(w, h, FilterType::CatmullRom)
}

// Convert a hex code to a color.
pub fn hex_to_color(hex: &str) -> Result<Rgba<u8>> {
    let hex_code = hex.strip_prefix('#').map_or(hex, |stripped| stripped);
    ensure!(hex_code.len() == 6, "Invalid hex code length");

    let red = u8::from_str_radix(&hex_code[..2], 16).context("Invalid hex code for red channel")?;
    let green =
        u8::from_str_radix(&hex_code[2..4], 16).context("Invalid hex code for green channel")?;
    let blue =
        u8::from_str_radix(&hex_code[4..6], 16).context("Invalid hex code for blue channel")?;

    Ok(Rgba([red, green, blue, 255]))
}

fn main() -> Result<()> {
    env_logger::init();
    let app = App::parse();

    info!("Opening images");
    // We need to read all the images before we can create the model.dd
    let mut images: Vec<DynamicImage> = app
        .image_paths
        .iter()
        .map(|path| image::open(path).unwrap())
        .collect();

    info!("Setting the global image dimensions");
    // If the user didn't specify the width or height, then we use the width
    // and height of the first image.
    let image_width = app.image_width.unwrap_or(images[0].width());
    let image_height = app.image_height.unwrap_or(images[0].height());

    info!("Resizing images if necessary");
    // Resize all the images to the same width (for portrait) or height (for
    // landscape).
    images = images
        .into_iter()
        .map(|image| prepare_image(&image, image_width, image_height, &app))
        .collect();

    // Create the model.
    let model = Model {
        images,
        image_width,
        image_height,
    };

    let n = app.image_paths.len() as u32;

    info!("Calculating the size of the output image");
    // Calculate the width and height of the output image.
    let (width, height) = match app.orientation {
        Orientation::Portrait => {
            let w = model.image_width + 2 * app.left_margin;
            let hs = model.images.iter().fold(0, |a, b| a + b.height());
            let h = hs + app.spacing * (n - 1) + 2 * app.top_margin;
            (w, h)
        }
        Orientation::Landscape => {
            let h = model.image_height + 2 * app.top_margin;
            let ws = model.images.iter().fold(0, |a, b| a + b.width());
            let w = ws + app.spacing * (n - 1) + 2 * app.left_margin;
            (w, h)
        }
    };

    info!(
        "Creating the blank output image with color {}",
        app.background_color
    );
    let mut out_image = RgbaImage::from_pixel(width, height, hex_to_color(&app.background_color)?);

    info!("Copying the images to the output image");
    // Copy the images to the output image.
    match app.orientation {
        Orientation::Portrait => {
            let x = app.left_margin;
            let mut y = app.top_margin;
            for image in &model.images {
                out_image.copy_from(image, x, y)?;
                y += image.height() + app.spacing;
            }
        }
        Orientation::Landscape => {
            let mut x = app.left_margin;
            let y = app.top_margin;
            for image in model.images {
                out_image.copy_from(&image, x, y)?;
                x += image.width() + app.spacing;
            }
        }
    }

    info!("Saving the output image");
    // Save the output image to the downloads dir as a png.
    let dirs = UserDirs::new().expect("Failed to get user dirs");
    let dir = dirs.download_dir().expect("Failed to get download dir");
    let path = format!(r"{}/{}", dir.to_string_lossy(), "collage");
    let mut num = 0;
    let mut sketch = PathBuf::from(format!(r"{path}_{num}"));
    sketch.set_extension("png");
    while sketch.exists() {
        num += 1;
        sketch = PathBuf::from(format!(r"{path}_{num}"));
        sketch.set_extension("png");
    }
    out_image.save(sketch)?;
    Ok(())
}
