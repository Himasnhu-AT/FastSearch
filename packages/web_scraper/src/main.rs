use anyhow::{Result, Context};
use std::path::PathBuf;
use web_scraper::WebScraper;
use std::env;
use tokio;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    
    let args: Vec<String> = env::args().collect();
    
    // Parse command-line arguments
    let mut config_path = None;
    let mut command = "scrape"; // Default command
    
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--config" | "-c" => {
                if i + 1 < args.len() {
                    config_path = Some(PathBuf::from(&args[i + 1]));
                    i += 2;
                } else {
                    return Err(anyhow::anyhow!("Missing config file path after --config"));
                }
            },
            "scrape" | "search" => {
                command = args[i].as_str();
                i += 1;
            },
            _ => {
                i += 1;
            }
        }
    }
    
    // Create the web scraper instance
    let scraper = WebScraper::new(config_path.as_deref())
        .context("Failed to initialize web scraper")?;
    
    // Execute the requested command
    match command {
        "scrape" => {
            println!("Starting web scraping process...");
            scraper.scrape_domains().await?;
            println!("Web scraping completed successfully!");
        },
        "search" => {
            if args.len() < 3 {
                return Err(anyhow::anyhow!("Missing search query"));
            }
            
            let query = &args[2];
            println!("Searching for: {}", query);
            
            let results = scraper.search(query)?;
            
            println!("Found {} results:", results.len());
            for (i, page) in results.iter().enumerate() {
                println!("{}. Title: {}", i + 1, page.title);
                println!("   URL: {}", page.url);
                println!("   Description: {}", page.get_description());
                println!();
            }
        },
        _ => {
            println!("Usage:");
            println!("  {} [--config <path>] scrape", args[0]);
            println!("  {} [--config <path>] search <query>", args[0]);
        }
    }
    
    Ok(())
}