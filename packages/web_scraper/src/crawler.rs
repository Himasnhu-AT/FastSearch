use crate::config::ScraperConfig;
use crate::models::PageData;
use crate::models::MetaTag;
use crate::parser;
use crate::stats::ScraperStats;
use anyhow::{Result, Context};
use reqwest::{Client, redirect};
use std::collections::{HashSet, HashMap};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tokio::time::sleep;
use url::Url;
use scraper::{Html, Selector};
use chrono::Utc;
use log::{info, debug};

#[derive(Clone)]
pub struct Crawler {
    config: ScraperConfig,
    client: Client,
    visited_urls: Arc<Mutex<HashSet<String>>>,
    domain_last_request: Arc<Mutex<HashMap<String, Instant>>>,
    stats: Arc<Mutex<ScraperStats>>,
}

impl Crawler {
    pub fn new(config: &ScraperConfig) -> Result<Self> {
        // Create HTTP client with configured settings
        let redirect_policy = if config.follow_redirects {
            redirect::Policy::limited(config.max_redirects)
        } else {
            redirect::Policy::none()
        };
        
        let client = Client::builder()
            .user_agent(&config.user_agent)
            .timeout(config.request_timeout())
            .redirect(redirect_policy)
            .build()
            .context("Failed to create HTTP client")?;
        
        Ok(Self {
            config: config.clone(),
            client,
            visited_urls: Arc::new(Mutex::new(HashSet::new())),
            domain_last_request: Arc::new(Mutex::new(HashMap::new())),
            stats: Arc::new(Mutex::new(ScraperStats::new())),
        })
    }
    
    /// Get a reference to the statistics tracker
    pub fn stats(&self) -> ScraperStats {
        self.stats.lock().unwrap().clone()
    }
    
    /// Process a domain by fetching sitemap
    pub async fn process_domain(&self, domain: &str) -> Result<Vec<String>> {
        let base_url = format!("https://{}", domain);
        let mut seed_urls = Vec::new();
        
        // Add the domain root URL
        seed_urls.push(base_url.clone());
        
        // Try common sitemap paths
        let sitemap_paths = [
            format!("{}/sitemap.xml", base_url),
            format!("{}/sitemap_index.xml", base_url),
            format!("{}/sitemap.txt", base_url),
        ];
        
        for sitemap_url in sitemap_paths {
            match self.process_sitemap(&sitemap_url).await {
                Ok(urls) => {
                    info!("Added {} URLs from sitemap {}", urls.len(), sitemap_url);
                    seed_urls.extend(urls);
                    break;
                }
                Err(e) => {
                    debug!("Failed to process sitemap {}: {}", sitemap_url, e);
                    self.stats.lock().unwrap().record_failure(&sitemap_url);
                }
            }
        }
        
        // Deduplicate URLs
        let unique_urls: HashSet<String> = seed_urls.into_iter().collect();
        
        Ok(unique_urls.into_iter().collect())
    }
    
    /// Scrape a single URL and extract its content
    pub async fn scrape_url(&self, url: &str) -> Result<PageData> {
        // Check if the URL has already been scraped
        if self.config.skip_already_scraped {
            let visited_urls = self.visited_urls.lock().unwrap();
            if visited_urls.contains(url) {
                return Err(anyhow::anyhow!("URL already scraped: {}", url).into());
            }
        }
        
        // Ensure we respect rate limits for the domain
        self.respect_rate_limit(url).await?;
        
        // Fetch the URL
        debug!("Fetching URL: {}", url);
        let response = self.client.get(url)
            .send()
            .await
            .with_context(|| format!("Failed to fetch URL: {}", url))?;
        
        let status = response.status();
        if !status.is_success() {
            self.stats.lock().unwrap().record_failure(url);
            return Err(anyhow::anyhow!("HTTP error: {} for URL: {}", status, url).into());
        }
        
        // Get the content type to ensure it's HTML
        let content_type = response.headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("text/html");
            
        if !content_type.contains("text/html") {
            self.stats.lock().unwrap().record_failure(url);
            return Err(anyhow::anyhow!("Not an HTML page: {} (Content-Type: {})", url, content_type).into());
        }
        
        // Extract HTML content
        let html = response.text().await
            .with_context(|| format!("Failed to read HTML content from: {}", url))?;
        
        // Record successful download with byte count
        self.stats.lock().unwrap().record_success(url, html.len());
        
        // Parse the HTML using the parser module
        let document = Html::parse_document(&html);
        
        // Extract page title
        let title_selector = Selector::parse("title").unwrap();
        let title = document.select(&title_selector)
            .next()
            .map(|element| element.inner_html().trim().to_string())
            .unwrap_or_else(|| "Untitled".to_string());
        
        // Extract meta tags
        let meta_selector = Selector::parse("meta").unwrap();
        let mut meta_tags = Vec::new();
        let mut canonical_url = None;
        
        for meta in document.select(&meta_selector) {
            let name = meta.value().attr("name").or_else(|| meta.value().attr("property"));
            let content = meta.value().attr("content");
            
            if let (Some(name), Some(content)) = (name, content) {
                meta_tags.push(MetaTag {
                    name: name.to_string(),
                    content: content.to_string(),
                });
            }
        }
        
        // Extract canonical URL
        let link_selector = Selector::parse("link[rel=canonical]").unwrap();
        if let Some(link) = document.select(&link_selector).next() {
            if let Some(href) = link.value().attr("href") {
                canonical_url = Some(href.to_string());
            }
        }
        
        // Extract main text content
        let content_text = parser::extract_main_content(&document);
        
        // Detect language
        let language = parser::detect_language(&content_text, &meta_tags);
        
        // Mark URL as visited
        {
            let mut visited_urls = self.visited_urls.lock().unwrap();
            visited_urls.insert(url.to_string());
        }
        
        Ok(PageData {
            url: url.to_string(),
            language,
            title,
            meta_tags,
            canonical_url,
            content_text,
            scraped_at: Utc::now(),
        })
    }
    
    /// Process a sitemap URL and extract all page URLs
    async fn process_sitemap(&self, sitemap_url: &str) -> Result<Vec<String>> {
        debug!("Processing sitemap: {}", sitemap_url);
        
        // Respect rate limits for the domain
        self.respect_rate_limit(sitemap_url).await?;
        
        // Fetch the sitemap
        let response = self.client.get(sitemap_url)
            .send()
            .await
            .with_context(|| format!("Failed to fetch sitemap: {}", sitemap_url))?;
        
        if !response.status().is_success() {
            return Err(anyhow::anyhow!("HTTP error: {} for sitemap: {}", response.status(), sitemap_url).into());
        }
        
        let content = response.text().await
            .with_context(|| format!("Failed to read sitemap content: {}", sitemap_url))?;
        
        // Extract URLs from the sitemap
        let mut urls = Vec::new();
        
        // Check if it's a sitemap index (contains other sitemaps)
        if content.contains("<sitemapindex") {
            debug!("Found sitemap index at {}", sitemap_url);
            let document = Html::parse_document(&content);
            let loc_selector = Selector::parse("sitemap > loc").unwrap();
            
            for loc in document.select(&loc_selector) {
                let loc_text = loc.inner_html().trim().to_string();
                
                // Use Box::pin to handle recursive async function calls
                let sub_urls = Box::pin(self.process_sitemap(&loc_text)).await?;
                urls.extend(sub_urls);
            }
        } else {
            // Regular sitemap
            let document = Html::parse_document(&content);
            let loc_selector = Selector::parse("url > loc").unwrap();
            
            for loc in document.select(&loc_selector) {
                let page_url = loc.inner_html().trim().to_string();
                urls.push(page_url);
            }
            
            // If XML parsing fails, try plain text format (one URL per line)
            if urls.is_empty() && !content.contains("<urlset") {
                for line in content.lines() {
                    let line = line.trim();
                    if !line.is_empty() && (line.starts_with("http://") || line.starts_with("https://")) {
                        urls.push(line.to_string());
                    }
                }
            }
        }
        
        debug!("Found {} URLs in sitemap {}", urls.len(), sitemap_url);
        Ok(urls)
    }
    
    /// Respect rate limits for a domain
    async fn respect_rate_limit(&self, url: &str) -> Result<()> {
        if let Ok(parsed_url) = Url::parse(url) {
            let host = match parsed_url.host_str() {
                Some(host) => host.to_string(),
                None => return Ok(()), // No host, no rate limit
            };
            
            let delay = {
                let domain_last_request = self.domain_last_request.lock().unwrap();
                
                if let Some(last_request) = domain_last_request.get(&host) {
                    let elapsed = last_request.elapsed();
                    let min_delay = self.config.request_delay();
                    
                    if elapsed < min_delay {
                        Some(min_delay - elapsed)
                    } else {
                        None
                    }
                } else {
                    None
                }
            };
            
            // Sleep if we need to respect the rate limit
            if let Some(delay) = delay {
                debug!("Rate limiting: sleeping for {:?} before requesting {}", delay, url);
                sleep(delay).await;
            }
            
            // Update the last request time
            let mut domain_last_request = self.domain_last_request.lock().unwrap();
            domain_last_request.insert(host, Instant::now());
        }
        
        Ok(())
    }
}