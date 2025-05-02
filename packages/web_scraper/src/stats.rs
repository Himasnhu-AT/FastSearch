use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use chrono::{DateTime, Utc};

/// Scraper statistics tracker
#[derive(Debug, Clone)]
pub struct ScraperStats {
    pub start_time: DateTime<Utc>,
    pub urls_processed: Arc<Mutex<usize>>,
    pub urls_failed: Arc<Mutex<usize>>,
    pub bytes_downloaded: Arc<Mutex<usize>>,
    pub domain_stats: Arc<Mutex<HashMap<String, DomainStat>>>,
}

/// Statistics for a specific domain
#[derive(Debug, Clone)]
pub struct DomainStat {
    pub domain: String,
    pub urls_processed: usize,
    pub urls_failed: usize,
    pub bytes_downloaded: usize,
    pub last_crawled: DateTime<Utc>,
}

impl ScraperStats {
    /// Create a new statistics tracker
    pub fn new() -> Self {
        Self {
            start_time: Utc::now(),
            urls_processed: Arc::new(Mutex::new(0)),
            urls_failed: Arc::new(Mutex::new(0)),
            bytes_downloaded: Arc::new(Mutex::new(0)),
            domain_stats: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    /// Record a successful URL scrape
    pub fn record_success(&self, url: &str, bytes: usize) {
        let domain = Self::extract_domain(url);
        
        // Update total stats
        {
            let mut processed = self.urls_processed.lock().unwrap();
            *processed += 1;
            
            let mut bytes_downloaded = self.bytes_downloaded.lock().unwrap();
            *bytes_downloaded += bytes;
        }
        
        // Update domain-specific stats
        {
            let mut domain_stats = self.domain_stats.lock().unwrap();
            let stat = domain_stats.entry(domain.clone()).or_insert_with(|| DomainStat {
                domain: domain.clone(),
                urls_processed: 0,
                urls_failed: 0,
                bytes_downloaded: 0,
                last_crawled: Utc::now(),
            });
            
            stat.urls_processed += 1;
            stat.bytes_downloaded += bytes;
            stat.last_crawled = Utc::now();
        }
    }
    
    /// Record a failed URL scrape
    pub fn record_failure(&self, url: &str) {
        let domain = Self::extract_domain(url);
        
        // Update total stats
        {
            let mut failed = self.urls_failed.lock().unwrap();
            *failed += 1;
        }
        
        // Update domain-specific stats
        {
            let mut domain_stats = self.domain_stats.lock().unwrap();
            let stat = domain_stats.entry(domain.clone()).or_insert_with(|| DomainStat {
                domain: domain.clone(),
                urls_processed: 0,
                urls_failed: 0,
                bytes_downloaded: 0,
                last_crawled: Utc::now(),
            });
            
            stat.urls_failed += 1;
            stat.last_crawled = Utc::now();
        }
    }
    
    /// Get total statistics as a formatted string
    pub fn get_stats_string(&self) -> String {
        let duration = Utc::now() - self.start_time;
        let urls_processed = *self.urls_processed.lock().unwrap();
        let urls_failed = *self.urls_failed.lock().unwrap();
        let bytes_downloaded = *self.bytes_downloaded.lock().unwrap();
        let mb_downloaded = bytes_downloaded as f64 / 1024.0 / 1024.0;
        
        let domain_stats = self.domain_stats.lock().unwrap();
        let domains_count = domain_stats.len();
        
        let mut result = format!(
            "Scraper Statistics:\n\
             ------------------\n\
             Runtime: {} seconds\n\
             Domains processed: {}\n\
             URLs processed: {}\n\
             URLs failed: {}\n\
             Data downloaded: {:.2} MB\n\
             \n\
             Domain-specific statistics:\n\
             ---------------------------\n",
            duration.num_seconds(),
            domains_count,
            urls_processed,
            urls_failed,
            mb_downloaded
        );
        
        // Add domain-specific stats
        for (i, (domain, stat)) in domain_stats.iter().enumerate().take(10) {
            let domain_mb = stat.bytes_downloaded as f64 / 1024.0 / 1024.0;
            result.push_str(&format!(
                "{}. {} - {} URLs processed, {} failed, {:.2} MB downloaded\n",
                i + 1,
                domain,
                stat.urls_processed,
                stat.urls_failed,
                domain_mb
            ));
        }
        
        if domains_count > 10 {
            result.push_str(&format!("... and {} more domains\n", domains_count - 10));
        }
        
        result
    }
    
    /// Extract domain from URL
    fn extract_domain(url: &str) -> String {
        if let Ok(parsed) = url::Url::parse(url) {
            if let Some(host) = parsed.host_str() {
                return host.to_string();
            }
        }
        "unknown".to_string()
    }
}