pub mod ollama;
pub mod whisper;
pub mod personas;

pub use ollama::{generate_response, OllamaClient};
pub use whisper::{transcribe_audio, WhisperClient};
pub use personas::{generate_random_persona, generate_persona_by_name, list_archetypes, ARCHETYPES};
