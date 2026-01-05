use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

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


impl LlmClient {
    pub fn new(ollama_url: String) -> Self {
        Self {
            client: Client::new(),
            url: ollama_url.into(),
        }
    }

    pub async fn generate(&self, model: &str, prompt: &str, temperature: f64, max_tokens: u32) -> Result<String, reqwest::Error> {
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

        match self
            .client
            .post(&request_url)
            .json(&request_body)
            .send()
            .await
        {
            Ok(res) => {
                let response_body = res.json::<GenerateResponse>().await?;
                let duration = start_time.elapsed();
                log::info!("LLM generation completed in {}ms", duration.as_millis());
                Ok(response_body.response)
            }
            Err(e) => {
                let duration = start_time.elapsed();
                log::error!("LLM generation failed after {}ms: {}", duration.as_millis(), e);
                Err(e)
            }
        }
    }

    pub async fn generate_embeddings(&self, model: &str, prompt: &str) -> Result<Vec<f64>, reqwest::Error> {
        let start_time = std::time::Instant::now();
        let request_url = format!("{}/api/embeddings", self.url);
        let request_body = EmbeddingRequest {
            model,
            prompt,
        };

        match self
            .client
            .post(&request_url)
            .json(&request_body)
            .send()
            .await
        {
            Ok(res) => {
                let response_body = res.json::<EmbeddingResponse>().await?;
                let duration = start_time.elapsed();
                log::info!("Embedding generation completed in {}ms", duration.as_millis());
                Ok(response_body.embedding)
            }
            Err(e) => {
                let duration = start_time.elapsed();
                log::error!("Embedding generation failed after {}ms: {}", duration.as_millis(), e);
                Err(e)
            }
        }
    }

    pub async fn check_health(&self) -> Result<bool, reqwest::Error> {
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
}
