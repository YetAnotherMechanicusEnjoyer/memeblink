use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct TextureFrame {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u32>,
}

#[derive(Debug, Clone)]
pub struct GifAnimation {
    frames: Vec<TextureFrame>,
    delays: Vec<Duration>,
    total_duration: Duration,
    start_time: Instant,
}

impl GifAnimation {
    pub fn new(frames: Vec<TextureFrame>, delays: Vec<Duration>) -> shared::error::Result<Self> {
        if frames.is_empty() {
            return Err(shared::error::MemeBlinkError::DecodeError(
                "GIF contains no frames".to_string(),
            ));
        }
        let total_duration = delays.iter().sum();
        Ok(Self {
            frames,
            delays,
            total_duration,
            start_time: Instant::now(),
        })
    }

    #[inline]
    pub fn reset(&mut self) {
        self.start_time = Instant::now();
    }

    pub fn current_frame(&self) -> &TextureFrame {
        if self.frames.len() == 1 {
            return &self.frames[0];
        }

        let elapsed = self.start_time.elapsed();
        let modulo_ms = if self.total_duration.as_millis() > 0 {
            elapsed.as_millis() % self.total_duration.as_millis()
        } else {
            0
        };

        let mut current_sum = 0;
        for (i, delay) in self.delays.iter().enumerate() {
            current_sum += delay.as_millis();
            if modulo_ms < current_sum {
                return &self.frames[i];
            }
        }
        &self.frames[0]
    }
}
