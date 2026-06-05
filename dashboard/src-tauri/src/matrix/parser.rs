use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct MemeConfig {
    pub image_url: Option<String>,
    pub text: Option<String>,
    pub duration_ms: u32,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub text_color: Option<String>,
}

impl Default for MemeConfig {
    fn default() -> Self {
        Self {
            image_url: None,
            text: None,
            duration_ms: 3000,
            width: None,
            height: None,
            text_color: Some("#ffffff".to_string()),
        }
    }
}

pub fn parse_matrix_message(content: &str) -> MemeConfig {
    let mut config = MemeConfig::default();
    let mut clean_text_words = Vec::new();
    let words: Vec<&str> = content.split_whitespace().collect();
    let mut i = 0;

    while i < words.len() {
        match words[i] {
            "-i" | "--image_url" if i + 1 < words.len() => {
                config.image_url = Some(words[i + 1].to_string());
                i += 2;
            }
            "-d" | "--duration" if i + 1 < words.len() => {
                if let Ok(val) = words[i + 1].parse::<u32>() {
                    config.duration_ms = val;
                }
                i += 2;
            }
            "-w" | "--width" if i + 1 < words.len() => {
                if let Ok(val) = words[i + 1].parse::<u32>() {
                    config.width = Some(val);
                }
                i += 2;
            }
            "-h" | "--height" if i + 1 < words.len() => {
                if let Ok(val) = words[i + 1].parse::<u32>() {
                    config.height = Some(val);
                }
                i += 2;
            }
            "-c" | "--color" if i + 1 < words.len() => {
                config.text_color = Some(words[i + 1].to_string());
                i += 2;
            }
            _ => {
                clean_text_words.push(words[i]);
                i += 1;
            }
        }
    }

    let final_text = clean_text_words.join(" ");
    if !final_text.is_empty() {
        config.text = Some(final_text);
    }

    config
}
