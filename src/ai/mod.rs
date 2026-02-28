pub mod ollama;
pub mod whisper;
pub mod personas;
pub mod rag;
pub mod search;

pub use ollama::{generate_response, OllamaClient};
pub use whisper::{transcribe_audio, WhisperClient};
pub use personas::{generate_random_persona, generate_persona_by_name, list_archetypes, ARCHETYPES};
pub use rag::{generate_embedding, store_memory, retrieve_memories, cleanup_old_memories, Memory};
pub use search::{search_web, should_search, format_search_results, SearchResult};
