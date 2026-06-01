use image::AnimationDecoder;
use image::codecs::gif::GifDecoder;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::time::Duration;

use crate::renderer::texture::{GifAnimation, TextureFrame};
use shared::error::{MemeBlinkError, Result};

pub fn decode_gif_file<P: AsRef<Path>>(path: P) -> Result<GifAnimation> {
    let path_str = path.as_ref().to_string_lossy().into_owned();

    let file = File::open(&path).map_err(|e| MemeBlinkError::IoError {
        path: path_str.clone(),
        source: e,
    })?;

    let reader = BufReader::new(file);

    let decoder =
        GifDecoder::new(reader).map_err(|e| MemeBlinkError::DecodeError(e.to_string()))?;

    let mut texture_frames = Vec::new();
    let mut frame_delays = Vec::new();

    for frame in decoder.into_frames() {
        let raw_frame = frame.map_err(|e| MemeBlinkError::DecodeError(e.to_string()))?;
        let (delay, texture) = process_single_frame(raw_frame);

        texture_frames.push(texture);
        frame_delays.push(delay);
    }

    GifAnimation::new(texture_frames, frame_delays)
}

fn process_single_frame(raw_frame: image::Frame) -> (Duration, TextureFrame) {
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
