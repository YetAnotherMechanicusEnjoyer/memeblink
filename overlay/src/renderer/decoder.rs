use image::codecs::gif::GifDecoder;
use image::{AnimationDecoder, GenericImageView, ImageFormat};
use std::fs;
use std::io::Cursor;
use std::path::Path;
use std::time::Duration;

use crate::renderer::texture::{MediaAsset, TextureFrame};
use shared::error::{MemeBlinkError, Result};
use shared::models::SizeConfig;

pub fn fetch_and_decode_asset(
    source: &str,
    req_width: Option<SizeConfig>,
    req_height: Option<SizeConfig>,
) -> Result<MediaAsset> {
    let raw_bytes = if source.starts_with("http://") || source.starts_with("https://") {
        log::info!("Downloading asset from URL: {}", source);
        reqwest::blocking::get(source)
            .map_err(|e| MemeBlinkError::IoError {
                path: source.to_string(),
                source: std::io::Error::other(e.to_string()),
            })?
            .bytes()
            .map_err(|e| MemeBlinkError::IoError {
                path: source.to_string(),
                source: std::io::Error::other(e.to_string()),
            })?
            .to_vec()
    } else {
        log::info!("Loading asset from local path: {}", source);
        let path = Path::new(source);
        fs::read(path).map_err(|e| MemeBlinkError::IoError {
            path: source.to_string(),
            source: e,
        })?
    };

    let cursor = Cursor::new(raw_bytes.clone());
    let format = image::guess_format(&raw_bytes)
        .map_err(|e| MemeBlinkError::DecodeError(format!("Unknown image format: {}", e)))?;

    if format == ImageFormat::Gif {
        let decoder =
            GifDecoder::new(cursor).map_err(|e| MemeBlinkError::DecodeError(e.to_string()))?;

        let mut texture_frames = Vec::new();
        let mut frame_delays = Vec::new();

        for frame_result in decoder.into_frames() {
            let raw_frame = frame_result.map_err(|e| MemeBlinkError::DecodeError(e.to_string()))?;
            let delay: Duration = raw_frame.delay().into();
            let buffer = raw_frame.into_buffer();

            let (orig_w, orig_h) = buffer.dimensions();

            let (target_w, target_h) = match (&req_width, &req_height) {
                (Some(SizeConfig::Fixed(w)), Some(SizeConfig::Fixed(h))) => (*w, *h),

                (Some(SizeConfig::Fixed(w)), _) => {
                    let h = ((*w as u64 * orig_h as u64) / orig_w as u64) as u32;
                    (*w, h.max(1))
                }

                (_, Some(SizeConfig::Fixed(h))) => {
                    let w = ((*h as u64 * orig_w as u64) / orig_h as u64) as u32;
                    (w.max(1), *h)
                }

                _ => (orig_w, orig_h),
            };

            let mut final_buffer = buffer;
            if target_w != orig_w || target_h != orig_h {
                let dyn_frame = image::DynamicImage::ImageRgba8(final_buffer);
                let resized_frame = dyn_frame.resize_exact(
                    target_w,
                    target_h,
                    image::imageops::FilterType::Triangle,
                );
                final_buffer = resized_frame.into_rgba8();
            }

            let texture = process_rgba_buffer(final_buffer);
            texture_frames.push(texture);
            frame_delays.push(delay);
        }

        MediaAsset::new_animated(texture_frames, frame_delays)
    } else {
        let mut dyn_image =
            image::load(cursor, format).map_err(|e| MemeBlinkError::DecodeError(e.to_string()))?;

        let (orig_w, orig_h) = dyn_image.dimensions();

        let (target_w, target_h) = match (&req_width, &req_height) {
            (Some(SizeConfig::Fixed(w)), Some(SizeConfig::Fixed(h))) => (*w, *h),
            (Some(SizeConfig::Fixed(w)), _) => {
                let h = ((*w as u64 * orig_h as u64) / orig_w as u64) as u32;
                (*w, h.max(1))
            }
            (_, Some(SizeConfig::Fixed(h))) => {
                let w = ((*h as u64 * orig_w as u64) / orig_h as u64) as u32;
                (w.max(1), *h)
            }
            _ => (orig_w, orig_h),
        };

        if target_w != orig_w || target_h != orig_h {
            dyn_image =
                dyn_image.resize_exact(target_w, target_h, image::imageops::FilterType::Triangle);
        }

        let texture = process_rgba_buffer(dyn_image.to_rgba8());
        Ok(MediaAsset::new_static(texture))
    }
}

fn process_rgba_buffer(rgba_image: image::RgbaImage) -> TextureFrame {
    let (width, height) = rgba_image.dimensions();
    let raw_bytes = rgba_image.into_raw();
    let mut pixels = Vec::with_capacity((width * height) as usize);

    for chunk in raw_bytes.chunks_exact(4) {
        let r = chunk[0] as u32;
        let g = chunk[1] as u32;
        let b = chunk[2] as u32;
        let a = chunk[3] as u32;
        let argb = (a << 24) | (r << 16) | (g << 8) | b;
        pixels.push(argb);
    }

    TextureFrame {
        width,
        height,
        pixels,
    }
}
