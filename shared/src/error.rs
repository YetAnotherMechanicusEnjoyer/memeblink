use thiserror::Error;

#[derive(Error, Debug)]
pub enum MemeBlinkError {
    #[error("Failed to initialize Wayland connection: {0}")]
    WaylandInitialization(String),

    #[error("IPC socket binding failed at {path}: {source}")]
    IpcBinding {
        path: String,
        #[source]
        source: std::io::Error,
    },

    #[error("File system I/O failure for path: {path}. Source: {source}")]
    IoError {
        path: String,
        #[source]
        source: std::io::Error,
    },

    #[error("Failed to decode image asset: {0}")]
    DecodeError(String),

    #[error("Failed to decode image from path: {0}")]
    ImageDecoding(String),

    #[error("Invalid configuration parameter: {0}")]
    InvalidConfiguration(String),
}

pub type Result<T> = std::result::Result<T, MemeBlinkError>;
