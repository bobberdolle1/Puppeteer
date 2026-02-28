use anyhow::{Context, Result};
use reqwest::Client;
use scraper::{Html, Selector};

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub title: String,
    pub snippet: String,
    pub url: String,
}

/// Search DuckDuckGo HTML interface and extract top results
pub async fn search_web(client: &Client, query: &str, max_results: usize) -> Result<Vec<SearchResult>> {
    let encoded_query = urlencoding::encode(query);
    let url = format!("https://html.duckduckgo.com/html/?q={}", encoded_query);

    let response = client
        .get(&url)
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .send()
        .await
        .context("Failed to send search request")?;

    let html = response
        .text()
        .await
        .context("Failed to get response text")?;

    parse_duckduckgo_results(&html, max_results)
}

/// Parse DuckDuckGo HTML results
fn parse_duckduckgo_results(html: &str, max_results: usize) -> Result<Vec<SearchResult>> {
    let document = Html::parse_document(html);
    
    // DuckDuckGo result selectors
    let result_selector = Selector::parse(".result").unwrap();
    let title_selector = Selector::parse(".result__a").unwrap();
    let snippet_selector = Selector::parse(".result__snippet").unwrap();
    
    let mut results = Vec::new();

    for element in document.select(&result_selector).take(max_results) {
        let title = element
            .select(&title_selector)
            .next()
            .map(|el| el.text().collect::<String>())
            .unwrap_or_default()
            .trim()
            .to_string();

        let snippet = element
            .select(&snippet_selector)
            .next()
            .map(|el| el.text().collect::<String>())
            .unwrap_or_default()
            .trim()
            .to_string();

        let url = element
            .select(&title_selector)
            .next()
            .and_then(|el| el.value().attr("href"))
            .unwrap_or_default()
            .to_string();

        if !title.is_empty() && !snippet.is_empty() {
            results.push(SearchResult {
                title,
                snippet,
                url,
            });
        }
    }

    Ok(results)
}

/// Check if a message requires web search using LLM
pub async fn should_search(
    client: &Client,
    ollama_url: &str,
    model: &str,
    message: &str,
) -> Result<Option<String>> {
    use serde_json::json;

    let prompt = format!(
        r#"Analyze this message and determine if it requires searching the internet for current facts, news, or real-time information.

Message: "{}"

If search is needed, reply ONLY with: SEARCH: <query>
If no search needed, reply ONLY with: NO

Examples:
- "what's the weather today?" → SEARCH: weather today
- "who won the game yesterday?" → SEARCH: game results yesterday
- "привет как дела?" → NO
- "что нового в мире?" → SEARCH: latest news
- "сколько стоит биткоин?" → SEARCH: bitcoin price

Your response:"#,
        message
    );

    let url = format!("{}/api/generate", ollama_url);
    let request = json!({
        "model": model,
        "prompt": prompt,
        "stream": false,
        "options": {
            "temperature": 0.1,
            "num_predict": 50
        }
    });

    let response = client
        .post(&url)
        .json(&request)
        .send()
        .await
        .context("Failed to send LLM request")?;

    let response_json: serde_json::Value = response
        .json()
        .await
        .context("Failed to parse LLM response")?;

    let response_text = response_json["response"]
        .as_str()
        .unwrap_or("")
        .trim();

    if response_text.starts_with("SEARCH:") {
        let query = response_text
            .strip_prefix("SEARCH:")
            .unwrap_or("")
            .trim()
            .to_string();
        Ok(Some(query))
    } else {
        Ok(None)
    }
}

/// Format search results for LLM context
pub fn format_search_results(results: &[SearchResult]) -> String {
    let mut formatted = String::from("[Ты только что загуглил это и прочитал следующие результаты]\n\n");
    
    for (i, result) in results.iter().enumerate() {
        formatted.push_str(&format!(
            "{}. {}\n{}\n\n",
            i + 1,
            result.title,
            result.snippet
        ));
    }

    formatted
}
