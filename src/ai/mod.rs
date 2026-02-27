pub mod ollama;
pub mod whisper;

pub use ollama::{generate_response, OllamaClient};
pub use whisper::{transcribe_audio, WhisperClient};
