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
use indicatif::{ProgressBar, ProgressStyle, MultiProgress};

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
            // The MultiProgress will automatically manage the output of the progress bars
            // We just need to keep this thread alive while scraping is happening
            loop {
                thread::sleep(std::time::Duration::from_millis(100));
            }
        });
        
        // Process each domain
        for (domain_idx, domain) in domains.iter().enumerate() {
            let domain_start = Instant::now();
            domain_progress.set_message(format!("Processing {} ...", domain));
            
            println!("┌ Domain {}/{}: {}", domain_idx + 1, total_domains, domain);
            
            // Fetch sitemap/URLs
            let url_fetch_start = Instant::now();
            
            let allowed_urls = match self.crawler.process_domain(domain).await {
                Ok(urls) => {
                    let fetch_duration = url_fetch_start.elapsed();
                    println!("├─ Found {} URLs in {:.2}s", urls.len(), fetch_duration.as_secs_f64());
                    urls
                },
                Err(e) => {
                    println!("├─ Error processing domain {}: {}", domain, e);
                    domain_progress.inc(1);
                    continue;
                }
            };
            
            if allowed_urls.is_empty() {
                println!("├─ No URLs found for domain {}", domain);
                println!("└ Completed in 0s");
                domain_progress.inc(1);
                continue;
            }
            
            // Create URL progress bar
            let url_count = allowed_urls.len();
            let url_progress = multi_progress.add(ProgressBar::new(url_count as u64));
            url_progress.set_style(
                ProgressStyle::default_bar()
                    .template("[{elapsed_precise}] {bar:40.green/yellow} {pos}/{len} URLs • {msg}")
                    .unwrap()
                    .progress_chars("=>-")
            );
            url_progress.set_message(format!("Scraping {}", domain));
            
            // Track successful and failed URLs
            let mut success_count = 0;
            let mut failure_count = 0;
            
            // Scrape each allowed URL
            for (url_idx, url) in allowed_urls.iter().enumerate() {
                let url_start = Instant::now();
                url_progress.set_message(format!("({}/{}) {}", url_idx + 1, url_count, url));
                
                match self.crawler.scrape_url(url).await {
                    Ok(page_data) => {
                        let elapsed = url_start.elapsed();
                        success_count += 1;
                        
                        // Show success info
                        println!("├─ ✓ [{}] {} ({}KB, {:.2}s)", 
                            url_idx + 1,
                            url, 
                            page_data.content_text.len() / 1024,
                            elapsed.as_secs_f64()
                        );
                        
                        // Store the page
                        if let Err(e) = self.storage.store_page(page_data) {
                            println!("├─── Storage error: {}", e);
                        }
                    },
                    Err(e) => {
                        println!("├─ ✗ [{}] {}: {}", url_idx + 1, url, e);
                        failure_count += 1;
                    }
                }
                
                url_progress.inc(1);
                
                // Respect rate limiting
                let delay = self.config.request_delay();
                thread::sleep(delay);
            }
            
            let domain_elapsed = domain_start.elapsed();
            println!("├─ Summary: {} successful, {} failed, took {:.2}s", 
                success_count, failure_count, domain_elapsed.as_secs_f64());
            println!("└ Completed domain {}", domain);
            println!();
            
            url_progress.finish_with_message(format!("Completed {} URLs from {}", url_count, domain));
            domain_progress.inc(1);
        }
        
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