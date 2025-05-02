use thiserror::Error;
use std::io;

#[derive(Error, Debug)]
pub enum ScraperError {
    #[error("HTTP error: {0}")]
    HttpError(#[from] reqwest::Error),
    
    #[error("IO error: {0}")]
    IoError(#[from] io::Error),
    
    #[error("URL parsing error: {0}")]
    UrlParseError(#[from] url::ParseError),
    
    #[error("Database error: {0}")]
    DatabaseError(#[from] rusqlite::Error),
    
    #[error("Serialization error: {0}")]
    SerdeError(#[from] serde_json::Error),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    #[error("Robots.txt error: {0}")]
    RobotsError(String),
    
    #[error("Sitemap error: {0}")]
    SitemapError(String),
    
    #[error("Scraping error: {0}")]
    ScrapingError(String),
    
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
    
    #[error("Unknown error: {0}")]
    Unknown(String),
}