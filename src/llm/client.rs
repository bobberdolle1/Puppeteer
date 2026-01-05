use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;

#[derive(Clone)]
pub struct LlmClient {
    client: Client,
    url: Arc<str>,
}

#[derive(Serialize)]
struct GenerateRequest<'a> {
    model: &'a str,
    prompt: &'a str,
    stream: bool,
    options: GenerateOptions,
}

#[derive(Serialize)]
struct GenerateOptions {
    temperature: f64,
    num_predict: u32,
}

#[derive(Deserialize)]
struct GenerateResponse {
    response: String,
}

#[derive(Serialize)]
struct EmbeddingRequest<'a> {
    model: &'a str,
    prompt: &'a str,
}

#[derive(Deserialize)]
struct EmbeddingResponse {
    embedding: Vec<f64>,
}

/// Vision request for multimodal models
#[derive(Serialize)]
struct VisionRequest<'a> {
    model: &'a str,
    prompt: &'a str,
    images: Vec<String>, // Base64 encoded images
    stream: bool,
    options: GenerateOptions,
}

/// Error types for LLM operations
#[derive(Debug)]
pub enum LlmError {
    Network(reqwest::Error),
    Timeout,
    QueueFull,
    InvalidResponse(String),
}

impl std::fmt::Display for LlmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LlmError::Network(e) => write!(f, "Network error: {}", e),
            LlmError::Timeout => write!(f, "Request timed out"),
            LlmError::QueueFull => write!(f, "Queue is full, try again later"),
            LlmError::InvalidResponse(msg) => write!(f, "Invalid response: {}", msg),
        }
    }
}

impl std::error::Error for LlmError {}

impl From<reqwest::Error> for LlmError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            LlmError::Timeout
        } else {
            LlmError::Network(err)
        }
    }
}

impl LlmClient {
    pub fn new(ollama_url: String) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(180))
            .connect_timeout(Duration::from_secs(10))
            .build()
            .unwrap_or_else(|_| Client::new());
        
        Self {
            client,
            url: ollama_url.into(),
        }
    }

    pub async fn generate(&self, model: &str, prompt: &str, temperature: f64, max_tokens: u32) -> Result<String, LlmError> {
        let start_time = std::time::Instant::now();
        let request_url = format!("{}/api/generate", self.url);
        let request_body = GenerateRequest {
            model,
            prompt,
            stream: false,
            options: GenerateOptions {
                temperature,
                num_predict: max_tokens,
            },
        };

        let response = self
            .client
            .post(&request_url)
            .json(&request_body)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            log::error!("LLM API error: {} - {}", status, body);
            return Err(LlmError::InvalidResponse(format!("HTTP {}: {}", status, body)));
        }

        let response_body = response.json::<GenerateResponse>().await?;
        let duration = start_time.elapsed();
        log::info!("LLM generation completed in {}ms", duration.as_millis());
        Ok(response_body.response)
    }

    /// Generate with timeout wrapper
    pub async fn generate_with_timeout(
        &self,
        model: &str,
        prompt: &str,
        temperature: f64,
        max_tokens: u32,
        timeout_secs: u64,
    ) -> Result<String, LlmError> {
        match tokio::time::timeout(
            Duration::from_secs(timeout_secs),
            self.generate(model, prompt, temperature, max_tokens),
        )
        .await
        {
            Ok(result) => result,
            Err(_) => {
                log::error!("LLM generation timed out after {}s", timeout_secs);
                Err(LlmError::Timeout)
            }
        }
    }

    /// Generate response for image (vision model)
    pub async fn generate_vision(
        &self,
        model: &str,
        prompt: &str,
        images_base64: Vec<String>,
        temperature: f64,
        max_tokens: u32,
    ) -> Result<String, LlmError> {
        let start_time = std::time::Instant::now();
        let request_url = format!("{}/api/generate", self.url);
        let request_body = VisionRequest {
            model,
            prompt,
            images: images_base64,
            stream: false,
            options: GenerateOptions {
                temperature,
                num_predict: max_tokens,
            },
        };

        let response = self
            .client
            .post(&request_url)
            .json(&request_body)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            log::error!("Vision API error: {} - {}", status, body);
            return Err(LlmError::InvalidResponse(format!("HTTP {}: {}", status, body)));
        }

        let response_body = response.json::<GenerateResponse>().await?;
        let duration = start_time.elapsed();
        log::info!("Vision generation completed in {}ms", duration.as_millis());
        Ok(response_body.response)
    }

    pub async fn generate_embeddings(&self, model: &str, prompt: &str) -> Result<Vec<f64>, LlmError> {
        let start_time = std::time::Instant::now();
        let request_url = format!("{}/api/embeddings", self.url);
        let request_body = EmbeddingRequest {
            model,
            prompt,
        };

        let response = self
            .client
            .post(&request_url)
            .json(&request_body)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(LlmError::InvalidResponse(format!("HTTP {}: {}", status, body)));
        }

        let response_body = response.json::<EmbeddingResponse>().await?;
        let duration = start_time.elapsed();
        log::info!("Embedding generation completed in {}ms", duration.as_millis());
        Ok(response_body.embedding)
    }

    pub async fn check_health(&self) -> Result<bool, LlmError> {
        let start_time = std::time::Instant::now();
        let request_url = format!("{}/api/tags", self.url);

        match self.client.get(&request_url).send().await {
            Ok(response) => {
                let duration = start_time.elapsed();
                log::info!("Ollama health check completed in {}ms", duration.as_millis());
                Ok(response.status().is_success())
            }
            Err(e) => {
                let duration = start_time.elapsed();
                log::info!("Ollama health check failed after {}ms: {}", duration.as_millis(), e);
                Ok(false)
            }
        }
    }

    /// List available models
    pub async fn list_models(&self) -> Result<Vec<String>, LlmError> {
        let request_url = format!("{}/api/tags", self.url);
        
        let response = self.client.get(&request_url).send().await?;
        
        if !response.status().is_success() {
            return Ok(vec![]);
        }

        #[derive(Deserialize)]
        struct ModelsResponse {
            models: Vec<ModelInfo>,
        }

        #[derive(Deserialize)]
        struct ModelInfo {
            name: String,
        }

        let models: ModelsResponse = response.json().await?;
        Ok(models.models.into_iter().map(|m| m.name).collect())
    }
}
