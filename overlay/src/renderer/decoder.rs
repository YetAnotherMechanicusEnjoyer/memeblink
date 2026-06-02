use image::codecs::gif::GifDecoder;
use image::{AnimationDecoder, GenericImageView, ImageFormat};
use std::fs;
use std::io::Cursor;
use std::path::Path;

use crate::renderer::texture::{MediaAsset, TextureFrame};
use shared::error::{MemeBlinkError, Result};
use shared::models::SizeConfig;

fn load_system_font() -> Result<fontdue::Font> {
    let handle = font_kit::source::SystemSource::new()
        .select_best_match(
            &[font_kit::family_name::FamilyName::SansSerif],
            &font_kit::properties::Properties::new(),
        )
        .map_err(|e| MemeBlinkError::DecodeError(format!("Font error: {}", e)))?;

    let bytes = match handle {
        font_kit::handle::Handle::Path { path, .. } => fs::read(path).unwrap_or_default(),
        font_kit::handle::Handle::Memory { bytes, .. } => bytes.to_vec(),
    };
    fontdue::Font::from_bytes(bytes, fontdue::FontSettings::default())
        .map_err(|e| MemeBlinkError::DecodeError(e.to_string()))
}

fn blend_color(bg: u32, r: u32, g: u32, b: u32, alpha: u32) -> u32 {
    if alpha == 0 {
        return bg;
    }
    let bg_a = (bg >> 24) & 0xFF;
    let out_a = alpha + (bg_a * (255 - alpha) / 255);
    if out_a == 0 {
        return 0;
    }

    let out_r = (r * alpha + ((bg >> 16) & 0xFF) * (255 - alpha)) / 255;
    let out_g = (g * alpha + ((bg >> 8) & 0xFF) * (255 - alpha)) / 255;
    let out_b = (b * alpha + (bg & 0xFF) * (255 - alpha)) / 255;

    (out_a << 24) | (out_r << 16) | (out_g << 8) | out_b
}

fn apply_text(
    frame: TextureFrame,
    font: &fontdue::Font,
    text: &str,
    pos: Option<&str>,
    color: Option<&str>,
    size: Option<f32>,
) -> TextureFrame {
    let size = size.unwrap_or(28.0);
    let hex = color.unwrap_or("#FFFFFF").trim_start_matches('#');
    let c_val = u32::from_str_radix(hex, 16).unwrap_or(0xFFFFFF);
    let (r, g, b) = ((c_val >> 16) & 0xFF, (c_val >> 8) & 0xFF, c_val & 0xFF);

    let glyphs: Vec<_> = text.chars().map(|c| font.rasterize(c, size)).collect();
    let text_w: u32 = glyphs.iter().map(|(m, _)| m.advance_width as u32).sum();
    let text_h = size.ceil() as u32 + 15;

    let out_w = frame.width.max(text_w);
    let out_h = frame.height + text_h;
    let mut pixels = vec![0u32; (out_w * out_h) as usize];

    let img_x = (out_w - frame.width) / 2;
    let img_y = if pos.unwrap_or("below") == "above" {
        text_h
    } else {
        0
    };

    for row in 0..frame.height {
        let src = (row * frame.width) as usize;
        let dst = ((row + img_y) * out_w + img_x) as usize;
        pixels[dst..dst + frame.width as usize]
            .copy_from_slice(&frame.pixels[src..src + frame.width as usize]);
    }

    if text_w > 0 {
        let bg_y_start = if pos.unwrap_or("below") == "above" {
            0
        } else {
            frame.height
        };
        let bg_y_end = bg_y_start + text_h;

        let bg_x_start = ((out_w as i32 - text_w as i32) / 2 - 8).max(0);
        let bg_x_end = ((out_w as i32 - text_w as i32) / 2 + text_w as i32 + 8).min(out_w as i32);

        for y in bg_y_start..bg_y_end {
            for x in bg_x_start..bg_x_end {
                let idx = (y * out_w + x as u32) as usize;
                pixels[idx] = blend_color(pixels[idx], 0, 0, 0, 128);
            }
        }
    }

    let mut cx = (out_w as i32 - text_w as i32) / 2;
    let base_y = if pos.unwrap_or("below") == "above" {
        size as i32
    } else {
        frame.height as i32 + 15 + size as i32
    };

    for (m, bitmap) in glyphs {
        for y in 0..m.height {
            for x in 0..m.width {
                let alpha = bitmap[y * m.width + x] as u32;
                let (px, py) = (
                    cx + m.xmin + x as i32,
                    base_y - m.ymin - m.height as i32 + y as i32,
                );
                if px >= 0 && px < out_w as i32 && py >= 0 && py < out_h as i32 {
                    let idx = (py * out_w as i32 + px) as usize;
                    pixels[idx] = blend_color(pixels[idx], r, g, b, alpha);
                }
            }
        }
        cx += m.advance_width as i32;
    }
    TextureFrame {
        width: out_w,
        height: out_h,
        pixels,
    }
}

fn process_rgba_buffer(rgba_image: image::RgbaImage) -> TextureFrame {
    let (width, height) = rgba_image.dimensions();
    let raw_bytes = rgba_image.into_raw();
    let mut pixels = Vec::with_capacity((width * height) as usize);

    for chunk in raw_bytes.chunks_exact(4) {
        let (r, g, b, a) = (
            chunk[0] as u32,
            chunk[1] as u32,
            chunk[2] as u32,
            chunk[3] as u32,
        );
        pixels.push((a << 24) | (r << 16) | (g << 8) | b);
    }
    TextureFrame {
        width,
        height,
        pixels,
    }
}

pub fn fetch_and_decode_asset(
    source: &str,
    req_w: Option<SizeConfig>,
    req_h: Option<SizeConfig>,
    text: Option<&str>,
    text_pos: Option<&str>,
    text_color: Option<&str>,
    text_size: Option<f32>,
) -> Result<MediaAsset> {
    let raw_bytes = if source.starts_with("http://") || source.starts_with("https://") {
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
        fs::read(Path::new(source)).map_err(|e| MemeBlinkError::IoError {
            path: source.to_string(),
            source: e,
        })?
    };

    let cursor = Cursor::new(raw_bytes.clone());
    let format = image::guess_format(&raw_bytes)
        .map_err(|e| MemeBlinkError::DecodeError(format!("Unknown format: {}", e)))?;
    let font = if text.is_some() {
        Some(load_system_font()?)
    } else {
        None
    };

    if format == ImageFormat::Gif {
        let decoder =
            GifDecoder::new(cursor).map_err(|e| MemeBlinkError::DecodeError(e.to_string()))?;
        let (mut frames, mut delays) = (Vec::new(), Vec::new());

        for frame_res in decoder.into_frames() {
            let raw_frame = frame_res.map_err(|e| MemeBlinkError::DecodeError(e.to_string()))?;
            delays.push(raw_frame.delay().into());

            let mut buffer = raw_frame.into_buffer();
            let (orig_w, orig_h) = buffer.dimensions();
            let (tw, th) = calculate_scale(orig_w, orig_h, &req_w, &req_h);

            if tw != orig_w || th != orig_h {
                buffer = image::DynamicImage::ImageRgba8(buffer)
                    .resize_exact(tw, th, image::imageops::FilterType::Triangle)
                    .into_rgba8();
            }

            let mut texture = process_rgba_buffer(buffer);
            if let (Some(t), Some(f)) = (text, &font) {
                texture = apply_text(texture, f, t, text_pos, text_color, text_size);
            }
            frames.push(texture);
        }
        MediaAsset::new_animated(frames, delays)
    } else {
        let mut dyn_image =
            image::load(cursor, format).map_err(|e| MemeBlinkError::DecodeError(e.to_string()))?;
        let (orig_w, orig_h) = dyn_image.dimensions();
        let (tw, th) = calculate_scale(orig_w, orig_h, &req_w, &req_h);

        if tw != orig_w || th != orig_h {
            dyn_image = dyn_image.resize_exact(tw, th, image::imageops::FilterType::Triangle);
        }

        let mut texture = process_rgba_buffer(dyn_image.to_rgba8());
        if let (Some(t), Some(f)) = (text, &font) {
            texture = apply_text(texture, f, t, text_pos, text_color, text_size);
        }
        Ok(MediaAsset::new_static(texture))
    }
}

fn calculate_scale(
    orig_w: u32,
    orig_h: u32,
    req_w: &Option<SizeConfig>,
    req_h: &Option<SizeConfig>,
) -> (u32, u32) {
    match (req_w, req_h) {
        (Some(SizeConfig::Fixed(w)), Some(SizeConfig::Fixed(h))) => (*w, *h),
        (Some(SizeConfig::Fixed(w)), _) => (
            *w,
            ((*w as u64 * orig_h as u64) / orig_w as u64).max(1) as u32,
        ),
        (_, Some(SizeConfig::Fixed(h))) => (
            ((*h as u64 * orig_w as u64) / orig_h as u64).max(1) as u32,
            *h,
        ),
        _ => (orig_w, orig_h),
    }
}
