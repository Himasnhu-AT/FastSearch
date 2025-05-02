use std::time::{Duration, Instant};
use url::Url;
use std::collections::HashSet;

/// Simple rate limiter for respecting crawl delays
pub struct RateLimiter {
    last_request: Option<Instant>,
    delay: Duration,
}

impl RateLimiter {
    pub fn new(delay: Duration) -> Self {
        Self {
            last_request: None,
            delay,
        }
    }
    
    /// Wait if needed to respect the rate limit
    pub async fn wait(&mut self) {
        if let Some(last) = self.last_request {
            let elapsed = last.elapsed();
            if elapsed < self.delay {
                let wait_time = self.delay - elapsed;
                tokio::time::sleep(wait_time).await;
            }
        }
        
        self.last_request = Some(Instant::now());
    }
}

/// Extract the domain from a URL
pub fn extract_domain(url_str: &str) -> Option<String> {
    if let Ok(url) = Url::parse(url_str) {
        url.host_str().map(|s| s.to_string())
    } else {
        None
    }
}

/// Check if a URL is valid
pub fn is_valid_url(url_str: &str) -> bool {
    if let Ok(url) = Url::parse(url_str) {
        url.scheme() == "http" || url.scheme() == "https"
    } else {
        false
    }
}

/// Create a URL identifier (normalized form for comparisons)
pub fn normalize_url_for_comparison(url_str: &str) -> Option<String> {
    if let Ok(mut url) = Url::parse(url_str) {
        // Remove fragments
        url.set_fragment(None);
        
        // Normalize paths (e.g., remove trailing slash)
        let path = url.path();
        if path == "/" {
            url.set_path("");
        }
        
        Some(url.to_string())
    } else {
        None
    }
}

/// Keep track of visited URLs to avoid duplicates
pub struct VisitedTracker {
    visited: HashSet<String>,
}

impl VisitedTracker {
    pub fn new() -> Self {
        Self {
            visited: HashSet::new(),
        }
    }
    
    pub fn is_visited(&self, url: &str) -> bool {
        if let Some(normalized) = normalize_url_for_comparison(url) {
            self.visited.contains(&normalized)
        } else {
            false
        }
    }
    
    pub fn mark_visited(&mut self, url: &str) {
        if let Some(normalized) = normalize_url_for_comparison(url) {
            self.visited.insert(normalized);
        }
    }
    
    pub fn visited_count(&self) -> usize {
        self.visited.len()
    }
}