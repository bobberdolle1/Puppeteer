use reqwest::Client;
use serde::Deserialize;
use std::time::Duration;

/// Voice transcription client using Whisper API (OpenAI-compatible)
#[derive(Clone)]
pub struct VoiceClient {
    client: Client,
    whisper_url: String,
}

#[derive(Deserialize)]
struct TranscriptionResponse {
    text: String,
}

impl VoiceClient {
    pub fn new(whisper_url: String) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(60))
            .build()
            .unwrap_or_else(|_| Client::new());
        
        Self { client, whisper_url }
    }

    /// Transcribe audio using Whisper API (OpenAI-compatible endpoint)
    pub async fn transcribe(&self, audio_data: Vec<u8>, filename: &str) -> Result<String, VoiceError> {
        use reqwest::multipart::{Form, Part};
        
        let part = Part::bytes(audio_data)
            .file_name(filename.to_string())
            .mime_str("audio/ogg")
            .map_err(|e| VoiceError::InvalidFormat(e.to_string()))?;
        
        let form = Form::new()
            .part("file", part)
            .text("model", "whisper-1");

        let response = self.client
            .post(&format!("{}/v1/audio/transcriptions", self.whisper_url))
            .multipart(form)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(VoiceError::ApiError(format!("HTTP {}: {}", status, body)));
        }

        let result: TranscriptionResponse = response.json().await?;
        Ok(result.text)
    }

    /// Check if voice service is available
    pub async fn is_available(&self) -> bool {
        self.client
            .get(&format!("{}/v1/models", self.whisper_url))
            .send()
            .await
            .map(|r| r.status().is_success())
            .unwrap_or(false)
    }
}

impl Default for VoiceClient {
    fn default() -> Self {
        Self::new("http://localhost:8080".to_string())
    }
}

#[derive(Debug)]
pub enum VoiceError {
    Network(reqwest::Error),
    ApiError(String),
    InvalidFormat(String),
}

impl std::fmt::Display for VoiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VoiceError::Network(e) => write!(f, "Network error: {}", e),
            VoiceError::ApiError(msg) => write!(f, "API error: {}", msg),
            VoiceError::InvalidFormat(msg) => write!(f, "Invalid format: {}", msg),
        }
    }
}

impl std::error::Error for VoiceError {}

impl From<reqwest::Error> for VoiceError {
    fn from(err: reqwest::Error) -> Self {
        VoiceError::Network(err)
    }
}
