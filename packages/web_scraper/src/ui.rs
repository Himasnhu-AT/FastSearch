use crate::models::PageData;
use crate::WebScraper;
use std::sync::Arc;
use warp::{Filter, Reply};
use serde::Serialize;
use serde_json::json;
use std::net::SocketAddr;

/// UI server for displaying scraper results
pub struct UiServer {
    scraper: Arc<WebScraper>,
    addr: String,
}

#[derive(Serialize)]
struct PageView {
    url: String,
    title: String,
    language: String,
    description: String,
    scraped_at: String,
    content_preview: String,
}

impl From<PageData> for PageView {
    fn from(page: PageData) -> Self {
        let content_preview = if page.content_text.len() > 200 {
            format!("{}...", &page.content_text[..200])
        } else {
            page.content_text.clone()
        };
        
        // Get values before they're moved
        let url = page.url.clone();
        let title = page.title.clone();
        let language = page.language.clone();
        let description = page.get_description();
        let scraped_at = page.scraped_at.to_rfc3339();
        
        Self {
            url,
            title,
            language,
            description,
            scraped_at,
            content_preview,
        }
    }
}

impl UiServer {
    /// Create a new UI server
    pub fn new(scraper: Arc<WebScraper>, addr: String) -> Self {
        Self {
            scraper,
            addr,
        }
    }
    
    /// Start the UI server
    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let scraper_clone = self.scraper.clone();
        
        // Define routes
        let api = warp::path("api");
        
        // Stats route
        let stats_route = api
            .and(warp::path("stats"))
            .and(warp::get())
            .and(with_scraper(scraper_clone.clone()))
            .map(|scraper| handle_stats(scraper));
        
        // Pages route
        let pages_route = api
            .and(warp::path("pages"))
            .and(warp::get())
            .and(with_scraper(scraper_clone.clone()))
            .map(|scraper| handle_pages(scraper));
        
        // Single page route
        let page_route = api
            .and(warp::path("page"))
            .and(warp::path::param::<String>())
            .and(warp::get())
            .and(with_scraper(scraper_clone.clone()))
            .map(|url, scraper| handle_page(url, scraper));
        
        // Search route
        let search_route = api
            .and(warp::path("search"))
            .and(warp::query::<SearchQuery>())
            .and(warp::get())
            .and(with_scraper(scraper_clone.clone()))
            .map(|query, scraper| handle_search(query, scraper));
        
        // Serve static files for the frontend
        let static_routes = warp::path::end()
            .and(warp::get())
            .and(warp::fs::file("./ui/index.html"))
            .or(warp::path("static")
                .and(warp::fs::dir("./ui/static")));
        
        // Combine all routes
        let routes = stats_route
            .or(pages_route)
            .or(page_route)
            .or(search_route)
            .or(static_routes);
        
        println!("Starting UI server at http://{}", self.addr);
        
        // Parse the socket address
        let socket_addr: SocketAddr = self.addr.parse()
            .map_err(|e| format!("Failed to parse address: {}", e))?;
            
        warp::serve(routes).run(socket_addr).await;
        
        Ok(())
    }
    
    /// Ensure static files exist
    pub fn ensure_static_files(&self) -> std::io::Result<()> {
        use std::fs;
        use std::path::Path;
        
        let ui_dir = Path::new("./ui");
        let static_dir = Path::new("./ui/static");
        
        // Create directories if they don't exist
        if !ui_dir.exists() {
            fs::create_dir(ui_dir)?;
        }
        
        if !static_dir.exists() {
            fs::create_dir(static_dir)?;
        }
        
        // Create index.html if it doesn't exist
        if !Path::new("./ui/index.html").exists() {
            fs::write("./ui/index.html", include_str!("../ui/index.html"))?;
        }
        
        // Create CSS file if it doesn't exist
        if !Path::new("./ui/static/style.css").exists() {
            fs::write("./ui/static/style.css", include_str!("../ui/static/style.css"))?;
        }
        
        // Create JS file if it doesn't exist
        if !Path::new("./ui/static/app.js").exists() {
            fs::write("./ui/static/app.js", include_str!("../ui/static/app.js"))?;
        }
        
        Ok(())
    }
}

// Helper function to pass the scraper to handlers
fn with_scraper(
    scraper: Arc<WebScraper>,
) -> impl Filter<Extract = (Arc<WebScraper>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || scraper.clone())
}

// Stats handler
fn handle_stats(scraper: Arc<WebScraper>) -> impl Reply {
    let stats = scraper.get_stats();
    warp::reply::json(&json!({ "stats": stats }))
}

// Pages handler
fn handle_pages(scraper: Arc<WebScraper>) -> impl Reply {
    match scraper.get_all_pages() {
        Ok(pages) => {
            let page_views: Vec<PageView> = pages.into_iter().map(PageView::from).collect();
            warp::reply::json(&page_views)
        }
        Err(_) => {
            warp::reply::json(&json!({ "error": "Failed to fetch pages" }))
        }
    }
}

// Single page handler
fn handle_page(url: String, scraper: Arc<WebScraper>) -> impl Reply {
    // URL will be URL-encoded, so decode it
    let decoded_url = match urlencoding::decode(&url) {
        Ok(decoded) => decoded.to_string(),
        Err(_) => url.clone(),
    };
    
    match scraper.get_page_by_url(&decoded_url) {
        Ok(Some(page)) => {
            warp::reply::json(&page)
        }
        Ok(None) => {
            warp::reply::json(&json!({ "error": "Page not found" }))
        }
        Err(_) => {
            warp::reply::json(&json!({ "error": "Failed to fetch page" }))
        }
    }
}

// Search query struct
#[derive(serde::Deserialize)]
struct SearchQuery {
    q: String,
}

// Search handler
fn handle_search(query: SearchQuery, scraper: Arc<WebScraper>) -> impl Reply {
    match scraper.search(&query.q) {
        Ok(pages) => {
            let page_views: Vec<PageView> = pages.into_iter().map(PageView::from).collect();
            warp::reply::json(&page_views)
        }
        Err(_) => {
            warp::reply::json(&json!({ "error": "Search failed" }))
        }
    }
}