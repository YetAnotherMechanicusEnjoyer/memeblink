use image::codecs::gif::GifDecoder;
use image::{AnimationDecoder, DynamicImage, ImageFormat};
use std::fs;
use std::io::Cursor;
use std::path::Path;
use std::time::Duration;

use crate::renderer::texture::{MediaAsset, TextureFrame};
use shared::error::{MemeBlinkError, Result};

fn process_gif_frame(raw_frame: image::Frame) -> (Duration, TextureFrame) {
    let delay: Duration = raw_frame.delay().into();
    let buffer = raw_frame.into_buffer();
    let (width, height) = buffer.dimensions();

    let raw_bytes = buffer.into_raw();
    let mut pixels = Vec::with_capacity((width * height) as usize);

    for chunk in raw_bytes.chunks_exact(4) {
        let r = chunk[0] as u32;
        let g = chunk[1] as u32;
        let b = chunk[2] as u32;
        let a = chunk[3] as u32;
        let argb = (a << 24) | (r << 16) | (g << 8) | b;
        pixels.push(argb);
    }

    (
        delay,
        TextureFrame {
            width,
            height,
            pixels,
        },
    )
}

fn process_static_image(dyn_image: DynamicImage) -> TextureFrame {
    let rgba_image = dyn_image.to_rgba8();
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

pub fn fetch_and_decode_asset(source: &str) -> Result<MediaAsset> {
    let raw_bytes = if source.starts_with("http://") || source.starts_with("https://") {
        log::info!("Downloading asset from URL: {source}");
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
        log::info!("Loading asset from local path: {source}");
        let path = Path::new(source);
        fs::read(path).map_err(|e| MemeBlinkError::IoError {
            path: source.to_string(),
            source: e,
        })?
    };

    let cursor = Cursor::new(raw_bytes.clone());

    let format = image::guess_format(&raw_bytes)
        .map_err(|e| MemeBlinkError::DecodeError(format!("Unknown image format: {e}")))?;

    if format == ImageFormat::Gif {
        let decoder =
            GifDecoder::new(cursor).map_err(|e| MemeBlinkError::DecodeError(e.to_string()))?;
        let mut texture_frames = Vec::new();
        let mut frame_delays = Vec::new();

        for frame_result in decoder.into_frames() {
            let raw_frame = frame_result.map_err(|e| MemeBlinkError::DecodeError(e.to_string()))?;
            let (delay, texture) = process_gif_frame(raw_frame);
            texture_frames.push(texture);
            frame_delays.push(delay);
        }

        MediaAsset::new_animated(texture_frames, frame_delays)
    } else {
        let dyn_image =
            image::load(cursor, format).map_err(|e| MemeBlinkError::DecodeError(e.to_string()))?;
        let texture = process_static_image(dyn_image);
        Ok(MediaAsset::new_static(texture))
    }
}
