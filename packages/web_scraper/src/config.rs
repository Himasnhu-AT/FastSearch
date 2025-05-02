use std::path::Path;
use std::fs;
use std::io::{self, BufRead};
use std::time::Duration;

/// Configuration for the web scraper
#[derive(Clone, Debug)]
pub struct ScraperConfig {
    /// User agent to identify the scraper
    pub user_agent: String,
    
    /// Maximum depth to crawl from seed URLs
    pub max_depth: usize,
    
    /// Maximum number of URLs to process per domain
    pub max_urls_per_domain: usize,
    
    /// Whether to follow redirects
    pub follow_redirects: bool,
    
    /// Maximum number of redirects to follow
    pub max_redirects: usize,
    
    /// Minimum delay between requests to the same domain (in milliseconds)
    pub rate_limit_ms: u64,
    
    /// Request timeout in seconds
    pub request_timeout_secs: u64,
    
    /// Skip URLs that have already been scraped
    pub skip_already_scraped: bool,
    
    /// Path to the database file
    pub database_path: String,
    
    /// Path to domains file
    pub domains_file: String,
}

impl Default for ScraperConfig {
    fn default() -> Self {
        Self {
            user_agent: format!("FastSearch-WebScraper/0.1.0 (fastitsearch@gmail.com)"),
            max_depth: 3,
            max_urls_per_domain: 1000,
            follow_redirects: true,
            max_redirects: 5,
            rate_limit_ms: 1000, // 1 second between requests to the same domain
            request_timeout_secs: 30,
            skip_already_scraped: true,
            database_path: "index.db".to_string(),
            domains_file: "domains.txt".to_string(),
        }
    }
}

impl ScraperConfig {
    /// Create a new configuration from a file path
    pub fn from_file(path: &Path) -> anyhow::Result<Self> {
        // Simple implementation - in a real app, we'd parse a config file
        // For now, just return the default config
        let config = Self::default();
        Ok(config)
    }
    
    /// Get the list of domains to scrape from the domains file
    pub fn get_domains(&self) -> anyhow::Result<Vec<String>> {
        let file = fs::File::open(&self.domains_file)?;
        let reader = io::BufReader::new(file);
        let mut domains = Vec::new();
        
        for line in reader.lines() {
            let line = line?;
            let trimmed = line.trim();
            
            // Skip empty lines and comments
            if !trimmed.is_empty() && !trimmed.starts_with('#') {
                domains.push(trimmed.to_string());
            }
        }
        
        Ok(domains)
    }
    
    pub fn request_timeout(&self) -> Duration {
        Duration::from_secs(self.request_timeout_secs)
    }
    
    pub fn request_delay(&self) -> Duration {
        Duration::from_millis(self.rate_limit_ms)
    }
}