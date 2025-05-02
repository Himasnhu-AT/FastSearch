pub mod crawler;
pub mod parser;
pub mod storage;
pub mod models;
pub mod errors;
pub mod config;
pub mod utils;

use std::path::Path;
use anyhow::Result;

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
        log::info!("Starting scrape for {} domains", domains.len());
        
        for domain in domains {
            log::info!("Processing domain: {}", domain);
            
            // Fetch robots.txt and sitemap
            let allowed_urls = self.crawler.process_domain(&domain).await?;
            
            // Scrape each allowed URL
            for url in allowed_urls {
                match self.crawler.scrape_url(&url).await {
                    Ok(page_data) => {
                        log::info!("Successfully scraped: {}", url);
                        self.storage.store_page(page_data)?;
                    }
                    Err(e) => {
                        log::error!("Failed to scrape {}: {}", url, e);
                    }
                }
            }
        }
        
        Ok(())
    }
    
    pub fn search(&self, query: &str) -> Result<Vec<models::PageData>> {
        self.storage.search(query)
    }
}