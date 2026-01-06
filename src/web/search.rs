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
        // Generate multiple search queries for better coverage
        let queries = self.generate_search_queries(query);
        tracing::debug!(target: "web", "🔍 Search queries: {:?}", queries);
        
        let mut all_results: Vec<SearchResult> = Vec::new();
        let mut seen_snippets: std::collections::HashSet<String> = std::collections::HashSet::new();
        
        // Execute searches in parallel
        let futures: Vec<_> = queries.iter().map(|q| self.search(q)).collect();
        let results = futures::future::join_all(futures).await;
        
        for (i, result) in results.into_iter().enumerate() {
            match result {
                Ok(search_results) => {
                    tracing::debug!(target: "web", "Query {} returned {} results", i + 1, search_results.len());
                    for r in search_results {
                        // Deduplicate by snippet content (first 100 chars)
                        let snippet_key: String = r.snippet.chars().take(100).collect();
                        if !seen_snippets.contains(&snippet_key) {
                            seen_snippets.insert(snippet_key);
                            all_results.push(r);
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!(target: "web", "Query {} failed: {}", i + 1, e);
                }
            }
        }
        
        if all_results.is_empty() {
            tracing::debug!(target: "web", "No results found for: {}", query);
            return String::new();
        }
        
        crate::logging::log_web_search(query, all_results.len());
        
        let mut context = format!("### Результаты веб-поиска по '{}':\n\n", query);
        for (i, result) in all_results.into_iter().take(max_results).enumerate() {
            context.push_str(&format!(
                "{}. {}\n{}\n\n",
                i + 1,
                result.title,
                result.snippet
            ));
        }
        context
    }
    
    /// Generate multiple search query variations for better results
    fn generate_search_queries(&self, original: &str) -> Vec<String> {
        let mut queries = vec![original.to_string()];
        
        let text_lower = original.to_lowercase();
        
        // Add "что такое" prefix for definition-like queries
        if !text_lower.contains("что такое") && !text_lower.contains("what is") {
            if text_lower.starts_with("что ") || text_lower.starts_with("кто ") {
                // Already a question, keep as is
            } else if original.split_whitespace().count() <= 3 {
                // Short query - might be looking for definition
                queries.push(format!("{} это", original));
            }
        }
        
        // Add year for potentially time-sensitive queries
        let current_year = chrono::Local::now().format("%Y").to_string();
        let time_sensitive = ["цена", "курс", "стоимость", "price", "cost", 
                             "лучший", "best", "топ", "top", "рейтинг", "rating",
                             "новый", "new", "последний", "latest"];
        
        if time_sensitive.iter().any(|t| text_lower.contains(t)) {
            if !original.contains(&current_year) {
                queries.push(format!("{} {}", original, current_year));
            }
        }
        
        // Add "купить" for price queries
        if (text_lower.contains("цена") || text_lower.contains("стоимость") || text_lower.contains("сколько стоит"))
            && !text_lower.contains("купить") {
            queries.push(format!("{} купить", original));
        }
        
        // Add "обзор" for product queries
        let product_indicators = ["rtx", "gtx", "ryzen", "intel", "iphone", "samsung", "nvidia", "amd"];
        if product_indicators.iter().any(|p| text_lower.contains(p)) && !text_lower.contains("обзор") {
            queries.push(format!("{} обзор", original));
        }
        
        // Limit to 3 queries max to avoid rate limiting
        queries.truncate(3);
        queries
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
