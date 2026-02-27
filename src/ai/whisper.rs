use anyhow::{Context, Result};
use reqwest::multipart;
use serde::Deserialize;
use std::path::Path;

/// Whisper API client for audio transcription
pub struct WhisperClient {
    base_url: String,
    client: reqwest::Client,
}

impl WhisperClient {
    pub fn new(base_url: String) -> Self {
        Self {
            base_url,
            client: reqwest::Client::new(),
        }
    }

    /// Transcribe an audio file
    pub async fn transcribe(&self, audio_path: &Path) -> Result<String> {
        let url = format!("{}/v1/audio/transcriptions", self.base_url);

        // Read the audio file
        let file_bytes = tokio::fs::read(audio_path)
            .await
            .context("Failed to read audio file")?;

        let file_name = audio_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("audio.ogg");

        // Create multipart form
        let form = multipart::Form::new()
            .text("model", "whisper-1")
            .part(
                "file",
                multipart::Part::bytes(file_bytes)
                    .file_name(file_name.to_string())
                    .mime_str("audio/ogg")?,
            );

        let response = self
            .client
            .post(&url)
            .multipart(form)
            .send()
            .await
            .context("Failed to send request to Whisper API")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("Whisper API error {}: {}", status, error_text);
        }

        let result: WhisperResponse = response
            .json()
            .await
            .context("Failed to parse Whisper response")?;

        Ok(result.text)
    }
}

#[derive(Debug, Deserialize)]
struct WhisperResponse {
    text: String,
}

/// Transcribe audio file using Whisper API
pub async fn transcribe_audio(whisper_url: &str, audio_path: &Path) -> Result<String> {
    let client = WhisperClient::new(whisper_url.to_string());
    client.transcribe(audio_path).await
}
