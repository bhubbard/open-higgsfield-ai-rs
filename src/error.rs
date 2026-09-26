use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("API request failed with status {status}: {message}")]
    Api { status: u16, message: String },

    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("Serialization / JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Model not found: '{0}'")]
    ModelNotFound(String),

    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),

    #[error("Generation timed out after {attempts} attempts ({elapsed_secs}s)")]
    Timeout {
        attempts: usize,
        elapsed_secs: u64,
    },

    #[error("Generation failed: {0}")]
    GenerationFailed(String),

    #[error("Missing API key: please set OPEN_HIGGSFIELD_API_KEY, MUAPI_KEY, or pass --api-key")]
    MissingApiKey,

    #[error("Storyboard error: {0}")]
    Storyboard(String),
}

pub type Result<T> = std::result::Result<T, Error>;
