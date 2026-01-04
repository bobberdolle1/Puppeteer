use serde::Deserialize;

#[derive(Clone, Deserialize)]
pub struct Config {
    #[serde(default = "default_ollama_url")]
    pub ollama_url: String,
    pub teloxide_token: String,
    pub database_url: String,
    pub owner_id: u64,
}

fn default_ollama_url() -> String {
    "http://ollama:11434".to_string()
}

impl Config {
    pub fn from_env() -> Result<Self, envy::Error> {
        envy::from_env::<Config>()
    }
}
