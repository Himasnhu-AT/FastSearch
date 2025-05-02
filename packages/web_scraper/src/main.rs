use anyhow::{Result, Context};
use web_scraper::WebScraper;
use web_scraper::ui::UiServer;
use std::env;
use std::sync::Arc;
use env_logger::Builder;
use log::LevelFilter;
use std::io::Write;

#[tokio::main]
async fn main() -> Result<()> {
    // Configure custom logger with timestamp and colors
    let mut builder = Builder::from_default_env();
    builder
        .format(|buf, record| {
            let level_style = buf.default_styled_level(record.level());
            writeln!(
                buf,
                "[{}] {} {}",
                chrono::Local::now().format("%H:%M:%S"),
                level_style,
                record.args()
            )
        })
        .filter(None, LevelFilter::Info)
        .init();
    
    let args: Vec<String> = env::args().collect();
    
    if args.len() > 1 {
        match args[1].as_str() {
            "scrape" => {
                let scraper = WebScraper::new(None)
                    .context("Failed to initialize web scraper")?;
                
                println!("┌─────────────────────────────────────────┐");
                println!("│        STARTING WEB SCRAPER             │");
                println!("└─────────────────────────────────────────┘");
                
                let start_time = std::time::Instant::now();
                
                // Run the scraper with progress tracking
                scraper.scrape_domains().await?;
                
                let elapsed = start_time.elapsed();
                println!("┌─────────────────────────────────────────┐");
                println!("│        SCRAPING COMPLETED               │");
                println!("│                                         │");
                println!("│  Total time: {:.2}s                   │", 
                    elapsed.as_secs_f64());
                println!("└─────────────────────────────────────────┘");
                
                println!("\nStatistics:");
                println!("{}", scraper.get_stats());
            },
            "stats" => {
                let scraper = WebScraper::new(None)
                    .context("Failed to initialize web scraper")?;
                println!("┌─────────────────────────────────────────┐");
                println!("│        SCRAPING STATISTICS              │");
                println!("└─────────────────────────────────────────┘");
                println!("{}", scraper.get_stats());
            },
            "ui" => {
                let port = if args.len() > 2 { &args[2] } else { "3000" };
                let addr = format!("127.0.0.1:{}", port);
                
                println!("┌─────────────────────────────────────────┐");
                println!("│        STARTING UI SERVER               │");
                println!("│                                         │");
                println!("│  URL: http://{}                │", addr);
                println!("└─────────────────────────────────────────┘");
                
                let scraper = Arc::new(WebScraper::new(None)
                    .context("Failed to initialize web scraper")?);
                let ui_server = UiServer::new(scraper, addr);
                
                ui_server.ensure_static_files()
                    .context("Failed to ensure static files for UI server")?;
                
                // Convert the Box<dyn Error> to anyhow::Error with a context
                if let Err(err) = ui_server.run().await {
                    return Err(anyhow::anyhow!("Failed to run UI server: {}", err));
                }
            },
            "help" | "-h" | "--help" => {
                print_usage();
            },
            _ => {
                println!("Unknown command: {}", args[1]);
                print_usage();
            }
        }
    } else {
        print_usage();
    }
    
    Ok(())
}

fn print_usage() {
    println!("Web Scraper CLI");
    println!("Usage:");
    println!("  web_scraper_cli scrape      - Start the scraping process");
    println!("  web_scraper_cli stats       - Show current statistics");
    println!("  web_scraper_cli ui [port]   - Start the UI server (default port: 3000)");
    println!("  web_scraper_cli help        - Show this help message");
}