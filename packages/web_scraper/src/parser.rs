use crate::models::MetaTag;
use scraper::{Html, Selector};
use regex::Regex;
use lazy_static::lazy_static;

/// Extract the main content text from an HTML document
pub fn extract_main_content(document: &Html) -> String {
    // Try to extract content from main article elements first
    let mut content = String::new();
    
    // Selectors for common content containers, in order of preference
    let content_selectors = [
        "article", "main", ".main-content", "#content", ".content",
        "[role=main]", ".post-content", ".entry-content"
    ];
    
    for selector_str in content_selectors {
        if let Ok(selector) = Selector::parse(selector_str) {
            let elements = document.select(&selector);
            for element in elements {
                let text = element.text().collect::<Vec<_>>().join(" ");
                if !text.trim().is_empty() {
                    content.push_str(&text);
                    content.push(' ');
                }
            }
            
            if !content.trim().is_empty() {
                break;
            }
        }
    }
    
    // If we couldn't find content in typical content containers, get text from body
    if content.trim().is_empty() {
        if let Ok(body_selector) = Selector::parse("body") {
            if let Some(body) = document.select(&body_selector).next() {
                content = body.text().collect::<Vec<_>>().join(" ");
            }
        }
    }
    
    // Clean up the content
    clean_content(&content)
}

/// Clean content by removing excess whitespace and formatting
fn clean_content(content: &str) -> String {
    lazy_static! {
        static ref WHITESPACE_RE: Regex = Regex::new(r"\s+").unwrap();
    }
    
    // Replace multiple whitespace with a single space
    let cleaned = WHITESPACE_RE.replace_all(content, " ").to_string();
    
    // Trim and return
    cleaned.trim().to_string()
}

/// Detect the language of the content
pub fn detect_language(_content: &str, meta_tags: &[MetaTag]) -> String {
    // First try to get language from meta tags
    for meta in meta_tags {
        let name = meta.name.to_lowercase();
        if name == "content-language" || name == "language" || name == "lang" {
            return meta.content.clone();
        }
    }
    
    // TODO: Implement more sophisticated language detection
    // For now, just return "en" as the default
    "en".to_string()
}

/// Extract links from a document, useful for crawling
pub fn extract_links(document: &Html, base_url: &str) -> Vec<String> {
    let mut links = Vec::new();
    
    if let Ok(link_selector) = Selector::parse("a[href]") {
        for link in document.select(&link_selector) {
            if let Some(href) = link.value().attr("href") {
                // Attempt to normalize the URL
                if let Some(normalized) = normalize_url(href, base_url) {
                    links.push(normalized);
                }
            }
        }
    }
    
    links
}

/// Normalize a URL to absolute form
fn normalize_url(href: &str, base_url: &str) -> Option<String> {
    // Skip fragment-only URLs, javascript, mailto, etc.
    if href.starts_with('#') || href.starts_with("javascript:") || 
       href.starts_with("mailto:") || href.starts_with("tel:") {
        return None;
    }
    
    // Try to parse the URL
    match url::Url::parse(href) {
        Ok(url) => {
            // Already absolute
            if url.scheme() == "http" || url.scheme() == "https" {
                return Some(url.to_string());
            }
            None
        },
        Err(url::ParseError::RelativeUrlWithoutBase) => {
            // Relative URL, attempt to join with base URL
            match url::Url::parse(base_url) {
                Ok(base) => {
                    match base.join(href) {
                        Ok(full_url) => Some(full_url.to_string()),
                        Err(_) => None,
                    }
                },
                Err(_) => None,
            }
        },
        Err(_) => None,
    }
}