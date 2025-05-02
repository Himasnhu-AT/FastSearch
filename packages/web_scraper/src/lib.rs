pub mod crawler;
pub mod parser;
pub mod storage;
pub mod models;
pub mod errors;
pub mod config;
pub mod utils;
pub mod stats;
pub mod ui;

use std::path::Path;
use anyhow::Result;
use std::time::Instant;
use std::thread;
use std::sync::{Arc, Mutex};
use indicatif::{ProgressBar, ProgressStyle, MultiProgress};
use futures::stream::{StreamExt, FuturesUnordered};
use tokio::sync::Semaphore;
use std::collections::HashMap;

pub struct WebScraper {
    config: config::ScraperConfig,
    crawler: crawler::Crawler,
    storage: storage::Storage,
}

impl WebScraper {
    pub fn new(config_path: Option<&Path>) -> Result<Self> {
        let config = match config_path {
            Some(path) => config::ScraperConfig::from_file(path)?,
            None => config::ScraperConfig::default(),
        };
        
        let crawler = crawler::Crawler::new(&config)?;
        let storage = storage::Storage::new(&config)?;
        
        Ok(Self {
            config,
            crawler,
            storage,
        })
    }
    
    pub async fn scrape_domains(&self) -> Result<()> {
        let domains = self.config.get_domains()?;
        let total_domains = domains.len();
        
        println!("Found {} domains to scrape", total_domains);
        println!();
        
        // Setup progress bars
        let multi_progress = MultiProgress::new();
        let domain_progress = multi_progress.add(ProgressBar::new(total_domains as u64));
        domain_progress.set_style(
            ProgressStyle::default_bar()
                .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} domains • {msg}")
                .unwrap()
                .progress_chars("##-")
        );
        domain_progress.set_message("Starting...");
        
        // Using a separate thread for the progress bar rendering
        let _progress_handle = thread::spawn(move || {
            loop {
                thread::sleep(std::time::Duration::from_millis(100));
            }
        });
        
        // Create a shared counter for domains processed
        let domains_processed = Arc::new(Mutex::new(0));
        
        // Create a semaphore to limit the number of concurrent domains
        let domain_semaphore = Arc::new(Semaphore::new(self.config.concurrent_domains));
        
        // Process domains concurrently using a FuturesUnordered collection
        let mut domain_futures = FuturesUnordered::new();
        
        for (domain_idx, domain) in domains.iter().enumerate() {
            // Clone what we need for the async block
            let domain = domain.clone();
            let domain_semaphore = domain_semaphore.clone();
            let domain_progress = domain_progress.clone();
            let domains_processed = domains_processed.clone();
            let crawler = self.crawler.clone();
            let storage = self.storage.clone();
            let config = self.config.clone();
            let domain_idx = domain_idx;
            let total_domains = total_domains;
            
            // Add the domain processing future to our collection
            domain_futures.push(async move {
                // Acquire a permit from the semaphore to limit concurrency
                let _permit = domain_semaphore.acquire().await.unwrap();
                
                let domain_start = Instant::now();
                domain_progress.set_message(format!("Processing {} ...", domain));
                
                println!("┌ Domain {}/{}: {}", domain_idx + 1, total_domains, domain);
                
                // Fetch sitemap/URLs
                let url_fetch_start = Instant::now();
                
                let allowed_urls = match crawler.process_domain(&domain).await {
                    Ok(urls) => {
                        let fetch_duration = url_fetch_start.elapsed();
                        println!("├─ Found {} URLs in {:.2}s", urls.len(), fetch_duration.as_secs_f64());
                        urls
                    },
                    Err(e) => {
                        println!("├─ Error processing domain {}: {}", domain, e);
                        *domains_processed.lock().unwrap() += 1;
                        domain_progress.inc(1);
                        return;
                    }
                };
                
                if allowed_urls.is_empty() {
                    println!("├─ No URLs found for domain {}", domain);
                    println!("└ Completed in 0s");
                    *domains_processed.lock().unwrap() += 1;
                    domain_progress.inc(1);
                    return;
                }
                
                // Create URL progress bar
                let url_count = allowed_urls.len();
                let url_progress = Arc::new(ProgressBar::new(url_count as u64));
                url_progress.set_style(
                    ProgressStyle::default_bar()
                        .template("[{elapsed_precise}] {bar:40.green/yellow} {pos}/{len} URLs • {msg}")
                        .unwrap()
                        .progress_chars("=>-")
                );
                url_progress.set_message(format!("Scraping {}", domain));
                
                // Track successful and failed URLs
                let success_count = Arc::new(Mutex::new(0));
                let failure_count = Arc::new(Mutex::new(0));
                
                // Create a rate limiter for this domain
                let rate_limiter = Arc::new(Semaphore::new(config.concurrent_urls));
                
                // Process URLs concurrently within each domain
                let mut url_futures = FuturesUnordered::new();
                
                for (url_idx, url) in allowed_urls.iter().enumerate() {
                    let url = url.clone();
                    let rate_limiter = rate_limiter.clone();
                    let url_progress = url_progress.clone();
                    let success_count = success_count.clone();
                    let failure_count = failure_count.clone();
                    let crawler = crawler.clone();
                    let storage = storage.clone();
                    let delay = config.request_delay();
                    let url_idx = url_idx;
                    
                    url_futures.push(async move {
                        // Acquire a permit from the rate limiter
                        let _permit = rate_limiter.acquire().await.unwrap();
                        
                        let url_start = Instant::now();
                        url_progress.set_message(format!("({}/{}) {}", url_idx + 1, url_count, url));
                        
                        match crawler.scrape_url(&url).await {
                            Ok(page_data) => {
                                let elapsed = url_start.elapsed();
                                *success_count.lock().unwrap() += 1;
                                
                                // Show success info
                                println!("├─ ✓ [{}] {} ({}KB, {:.2}s)", 
                                    url_idx + 1,
                                    url, 
                                    page_data.content_text.len() / 1024,
                                    elapsed.as_secs_f64()
                                );
                                
                                // Store the page
                                if let Err(e) = storage.store_page(page_data) {
                                    println!("├─── Storage error: {}", e);
                                }
                            },
                            Err(e) => {
                                println!("├─ ✗ [{}] {}: {}", url_idx + 1, url, e);
                                *failure_count.lock().unwrap() += 1;
                            }
                        }
                        
                        url_progress.inc(1);
                        
                        // Respect rate limiting
                        tokio::time::sleep(delay).await;
                    });
                }
                
                // Wait for all URL futures to complete
                while let Some(_) = url_futures.next().await {}
                
                let domain_elapsed = domain_start.elapsed();
                println!("├─ Summary: {} successful, {} failed, took {:.2}s", 
                    *success_count.lock().unwrap(), 
                    *failure_count.lock().unwrap(), 
                    domain_elapsed.as_secs_f64());
                println!("└ Completed domain {}", domain);
                println!();
                
                url_progress.finish_with_message(format!("Completed {} URLs from {}", url_count, domain));
                
                *domains_processed.lock().unwrap() += 1;
                domain_progress.inc(1);
            });
        }
        
        // Wait for all domain futures to complete
        while let Some(_) = domain_futures.next().await {}
        
        domain_progress.finish_with_message(format!("All {} domains processed", total_domains));
        
        Ok(())
    }
    
    /// Get scraping statistics as a formatted string
    pub fn get_stats(&self) -> String {
        self.crawler.stats().get_stats_string()
    }
    
    /// Search for pages matching a query
    pub fn search(&self, query: &str) -> Result<Vec<models::PageData>> {
        self.storage.search(query)
    }
    
    /// Get all pages from the database
    pub fn get_all_pages(&self) -> Result<Vec<models::PageData>> {
        self.storage.get_all_pages()
    }
    
    /// Get a specific page by URL
    pub fn get_page_by_url(&self, url: &str) -> Result<Option<models::PageData>> {
        self.storage.get_page_by_url(url)
    }
    
    /// Export all pages to a JSON file
    pub fn export_to_json(&self, path: &Path) -> Result<()> {
        self.storage.export_to_json(path)
    }
}