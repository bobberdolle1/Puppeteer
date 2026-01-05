use reqwest::Client;
use serde::Deserialize;
use std::time::Duration;

/// Web search client using DuckDuckGo Instant Answer API
#[derive(Clone)]
pub struct WebSearchClient {
    client: Client,
}

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub title: String,
    pub snippet: String,
    pub url: String,
}

#[derive(Deserialize)]
struct DdgResponse {
    #[serde(rename = "Abstract")]
    abstract_text: Option<String>,
    #[allow(dead_code)]
    #[serde(rename = "AbstractSource")]
    abstract_source: Option<String>,
    #[serde(rename = "AbstractURL")]
    abstract_url: Option<String>,
    #[serde(rename = "Heading")]
    heading: Option<String>,
    #[serde(rename = "RelatedTopics")]
    related_topics: Option<Vec<DdgTopic>>,
    #[serde(rename = "Answer")]
    answer: Option<String>,
    #[serde(rename = "AnswerType")]
    answer_type: Option<String>,
}

#[derive(Deserialize)]
struct DdgTopic {
    #[serde(rename = "Text")]
    text: Option<String>,
    #[serde(rename = "FirstURL")]
    first_url: Option<String>,
}

impl WebSearchClient {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .user_agent("PersonaForge/1.0")
            .build()
            .unwrap_or_else(|_| Client::new());
        
        Self { client }
    }

    /// Search using DuckDuckGo Instant Answer API
    /// Returns abstract/answer and related topics
    pub async fn search(&self, query: &str) -> Result<Vec<SearchResult>, SearchError> {
        let url = format!(
            "https://api.duckduckgo.com/?q={}&format=json&no_html=1&skip_disambig=1",
            urlencoding::encode(query)
        );

        let response = self.client.get(&url).send().await?;
        
        if !response.status().is_success() {
            return Err(SearchError::ApiError(format!("HTTP {}", response.status())));
        }

        let ddg: DdgResponse = response.json().await?;
        let mut results = Vec::new();

        // Add direct answer if available
        if let Some(answer) = ddg.answer {
            if !answer.is_empty() {
                results.push(SearchResult {
                    title: ddg.answer_type.unwrap_or_else(|| "Answer".to_string()),
                    snippet: answer,
                    url: String::new(),
                });
            }
        }

        // Add abstract if available
        if let Some(abstract_text) = ddg.abstract_text {
            if !abstract_text.is_empty() {
                results.push(SearchResult {
                    title: ddg.heading.unwrap_or_else(|| query.to_string()),
                    snippet: abstract_text,
                    url: ddg.abstract_url.unwrap_or_default(),
                });
            }
        }

        // Add related topics
        if let Some(topics) = ddg.related_topics {
            for topic in topics.into_iter().take(3) {
                if let (Some(text), Some(url)) = (topic.text, topic.first_url) {
                    if !text.is_empty() {
                        results.push(SearchResult {
                            title: text.chars().take(50).collect::<String>() + "...",
                            snippet: text,
                            url,
                        });
                    }
                }
            }
        }

        Ok(results)
    }

    /// Search and format results for LLM context
    pub async fn search_for_context(&self, query: &str, max_results: usize) -> String {
        match self.search(query).await {
            Ok(results) if !results.is_empty() => {
                let mut context = format!("### Web search results for '{}':\n\n", query);
                for (i, result) in results.into_iter().take(max_results).enumerate() {
                    context.push_str(&format!(
                        "{}. {}\n{}\n\n",
                        i + 1,
                        result.title,
                        result.snippet
                    ));
                }
                context
            }
            Ok(_) => String::new(),
            Err(e) => {
                log::warn!("Web search failed: {}", e);
                String::new()
            }
        }
    }
}

impl Default for WebSearchClient {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub enum SearchError {
    Network(reqwest::Error),
    ApiError(String),
}

impl std::fmt::Display for SearchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SearchError::Network(e) => write!(f, "Network error: {}", e),
            SearchError::ApiError(msg) => write!(f, "API error: {}", msg),
        }
    }
}

impl std::error::Error for SearchError {}

impl From<reqwest::Error> for SearchError {
    fn from(err: reqwest::Error) -> Self {
        SearchError::Network(err)
    }
}

/// Detect if a message likely needs web search
pub fn needs_web_search(text: &str) -> bool {
    let text_lower = text.to_lowercase();
    
    // Keywords that suggest need for current info
    let search_triggers = [
        "погода", "weather",
        "новости", "news",
        "курс", "цена", "price", "стоимость",
        "сегодня", "today", "сейчас", "now",
        "последний", "latest", "новый", "new",
        "что такое", "what is", "кто такой", "who is",
        "как", "how to",
        "где", "where",
        "когда", "when",
        "почему", "why",
        "сколько", "how much", "how many",
        "найди", "найти", "search", "find",
        "загугли", "google",
        "актуальн", "current",
    ];

    search_triggers.iter().any(|trigger| text_lower.contains(trigger))
}
