use crate::{BacklightOpts, CpuOpts, RamOpts};
use fast_image_resize::{FilterType, PixelType, Resizer};
use image::RgbaImage;
use std::{error::Error, fs, num::NonZeroU32, process::Command};

pub fn new_command(command: &str, args: &str) -> Result<String, Box<dyn Error>> {
    Ok(String::from_utf8(
        Command::new(command)
            .args(args.split_whitespace())
            .output()?
            .stdout,
    )?
    .trim()
    .to_string())
}

pub fn get_ram(opt: RamOpts) -> Result<String, Box<dyn Error>> {
    let output = new_command("free", "-m")?;
    let output = output.split_whitespace().collect::<Vec<&str>>();
    let total = output[7].parse::<f64>()?;
    let used = output[8].parse::<f64>()?;

    Ok(match opt {
        RamOpts::PercUsed => (used / total) * 100.0,
        RamOpts::PercFree => ((total - used) / total) * 100.0,
        RamOpts::Used => used,
        RamOpts::Free => total - used,
    }
    .to_string())
}

pub fn get_backlight(opts: BacklightOpts) -> Result<String, Box<dyn Error>> {
    let brightness = fs::read_to_string("/sys/class/backlight/intel_backlight/actual_brightness")?
        .trim()
        .parse::<f64>()?;

    let max_brightness = fs::read_to_string("/sys/class/backlight/intel_backlight/max_brightness")?
        .trim()
        .parse::<f64>()?;

    match opts {
        BacklightOpts::Perc => Ok(((brightness / max_brightness) * 100.0).to_string()),
        BacklightOpts::Value => Ok(brightness.to_string()),
    }
}

pub fn get_cpu(opts: CpuOpts) -> Result<String, Box<dyn Error>> {
    let output = new_command("mpstat", "")?;
    let output = output.split_whitespace().collect::<Vec<&str>>();
    let idle = output.last().ok_or("not found")?.parse::<f64>()?;

    match opts {
        CpuOpts::Perc => Ok((100.0 - idle).to_string()),
    }
}

pub fn get_current_workspace(
    active: &'static str,
    inactive: &'static str,
) -> Result<String, Box<dyn Error>> {
    let workspaces = new_command("hyprctl", "workspaces -j")?;

    let active_workspace = Command::new("hyprctl")
        .args(["activeworkspace", "-j"])
        .output()?
        .stdout;
    let active_workspace = String::from_utf8(active_workspace)?;

    let active_workspace = serde_json::from_str::<serde_json::Value>(&active_workspace)?;
    let active_workspace = active_workspace.get("id").ok_or("")?.as_i64().ok_or("")? as usize - 1;

    let length = serde_json::from_str::<serde_json::Value>(&workspaces)?
        .as_array()
        .ok_or("")?
        .len();

    Ok((0..length)
        .map(|i| {
            if i == active_workspace || i == length - 1 && active_workspace >= length {
                format!("{} ", active)
            } else {
                format!("{} ", inactive)
            }
        })
        .collect::<String>())
}

pub fn resize_image(image: &RgbaImage, width: u32, height: u32) -> Result<Vec<u8>, Box<dyn Error>> {
    let (img_w, img_h) = image.dimensions();
    let image = image.as_raw().to_vec();

    if img_w == width && img_h == height {
        return Ok(image);
    }

    let ratio = width as f32 / height as f32;
    let img_r = img_w as f32 / img_h as f32;

    let (trg_w, trg_h) = if ratio > img_r {
        let scale = height as f32 / img_h as f32;
        ((img_w as f32 * scale) as u32, height)
    } else {
        let scale = width as f32 / img_w as f32;
        (width, (img_h as f32 * scale) as u32)
    };

    let trg_w = trg_w.min(width);
    let trg_h = trg_h.min(height);

    // If img_w, img_h, trg_w or trg_h is 0 you have bigger problems than unsafety
    let src = fast_image_resize::Image::from_vec_u8(
        unsafe { NonZeroU32::new_unchecked(img_w) },
        unsafe { NonZeroU32::new_unchecked(img_h) },
        image,
        PixelType::U8x4,
    )?;

    let new_w = unsafe { NonZeroU32::new_unchecked(trg_w) };
    let new_h = unsafe { NonZeroU32::new_unchecked(trg_h) };

    let mut dst = fast_image_resize::Image::new(new_w, new_h, PixelType::U8x3);
    let mut dst_view = dst.view_mut();

    let mut resizer = Resizer::new(fast_image_resize::ResizeAlg::Convolution(
        FilterType::Lanczos3,
    ));

    resizer.resize(&src.view(), &mut dst_view)?;

    let dst = dst.into_vec();
    Ok(dst)
}
