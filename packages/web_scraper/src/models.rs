use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Represents a scraped web page
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageData {
    /// The URL of the page
    pub url: String,
    
    /// The detected language of the page content
    pub language: String,
    
    /// The page title
    pub title: String,
    
    /// Meta tags from the page (name -> content)
    pub meta_tags: Vec<MetaTag>,
    
    /// Canonical URL if specified
    pub canonical_url: Option<String>,
    
    /// Main text content of the page
    pub content_text: String,
    
    /// When the page was scraped
    #[serde(with = "chrono::serde::ts_seconds")]
    pub scraped_at: DateTime<Utc>,
}

/// Represents a meta tag with name and content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaTag {
    pub name: String,
    pub content: String,
}

impl PageData {
    /// Get description from meta tags or a short excerpt from content
    pub fn get_description(&self) -> String {
        // First try to get description from meta tags
        for meta in &self.meta_tags {
            if meta.name.to_lowercase() == "description" {
                return meta.content.clone();
            }
        }
        
        // Fall back to short excerpt from content
        let mut excerpt = self.content_text.chars().take(150).collect::<String>();
        if self.content_text.len() > 150 {
            excerpt.push_str("...");
        }
        excerpt
    }
    
    /// Get keywords from meta tags
    pub fn get_keywords(&self) -> Vec<String> {
        for meta in &self.meta_tags {
            if meta.name.to_lowercase() == "keywords" {
                return meta.content
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
            }
        }
        Vec::new()
    }
}