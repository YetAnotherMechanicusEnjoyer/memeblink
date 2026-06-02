use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct TextureFrame {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u32>,
}

#[derive(Debug, Clone)]
pub enum MediaAsset {
    Static(TextureFrame),
    Animated {
        frames: Vec<TextureFrame>,
        delays: Vec<Duration>,
        total_duration: Duration,
        start_time: Instant,
    },
}

impl MediaAsset {
    pub fn new_static(frame: TextureFrame) -> Self {
        Self::Static(frame)
    }

    pub fn new_animated(
        frames: Vec<TextureFrame>,
        delays: Vec<Duration>,
    ) -> shared::error::Result<Self> {
        if frames.is_empty() {
            return Err(shared::error::MemeBlinkError::DecodeError(
                "Animation contains no frame".to_string(),
            ));
        }
        let total_duration = delays.iter().sum();
        Ok(Self::Animated {
            frames,
            delays,
            total_duration,
            start_time: Instant::now(),
        })
    }

    #[inline]
    pub fn reset(&mut self) {
        if let Self::Animated { start_time, .. } = self {
            *start_time = Instant::now();
        }
    }

    pub fn current_frame(&self) -> &TextureFrame {
        match self {
            Self::Static(frame) => frame,
            Self::Animated {
                frames,
                delays,
                total_duration,
                start_time,
            } => {
                if frames.len() == 1 {
                    return &frames[0];
                }

                let elapsed = start_time.elapsed();
                let modulo_ms = if total_duration.as_millis() > 0 {
                    elapsed.as_millis() % total_duration.as_millis()
                } else {
                    0
                };

                let mut current_sum = 0;
                for (i, delay) in delays.iter().enumerate() {
                    current_sum += delay.as_millis();
                    if modulo_ms < current_sum {
                        return &frames[i];
                    }
                }
                &frames[0]
            }
        }
    }
}
