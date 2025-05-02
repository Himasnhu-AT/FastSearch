use crate::config::ScraperConfig;
use crate::models::PageData;
use anyhow::{Result, Context};
use rusqlite::{Connection, params};
use std::path::Path;
use std::fs;
use serde_json;
use log::{warn, debug};
use std::sync::{Arc, Mutex};

pub struct Storage {
    // Using `#[allow(dead_code)]` to suppress the warning as we might use this field in the future
    #[allow(dead_code)]
    config: ScraperConfig,
    connection: Arc<Mutex<Connection>>,
}

impl Storage {
    pub fn new(config: &ScraperConfig) -> Result<Self> {
        // Create database directory if it doesn't exist
        if let Some(parent) = Path::new(&config.database_path).parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }
        
        // Open SQLite database connection
        let connection = Connection::open(&config.database_path)
            .with_context(|| format!("Failed to open database at {}", config.database_path))?;
        
        // Initialize the database schema
        Self::initialize_schema(&connection)?;
        
        Ok(Self {
            config: config.clone(),
            connection: Arc::new(Mutex::new(connection)),
        })
    }
    
    /// Initialize the database schema
    fn initialize_schema(connection: &Connection) -> Result<()> {
        connection.execute(
            "CREATE TABLE IF NOT EXISTS pages (
                url TEXT PRIMARY KEY,
                language TEXT NOT NULL,
                title TEXT NOT NULL,
                meta_tags TEXT NOT NULL,
                canonical_url TEXT,
                content_text TEXT NOT NULL,
                scraped_at TEXT NOT NULL
            )",
            [],
        )?;
        
        // Create full-text search index
        connection.execute(
            "CREATE VIRTUAL TABLE IF NOT EXISTS pages_fts USING fts5(
                url, 
                title, 
                content_text, 
                content='pages', 
                content_rowid='rowid'
            )",
            [],
        )?;
        
        // Create trigger to keep FTS index updated
        connection.execute(
            "CREATE TRIGGER IF NOT EXISTS pages_ai AFTER INSERT ON pages BEGIN
                INSERT INTO pages_fts(rowid, url, title, content_text) 
                VALUES (new.rowid, new.url, new.title, new.content_text);
            END",
            [],
        )?;
        
        connection.execute(
            "CREATE TRIGGER IF NOT EXISTS pages_ad AFTER DELETE ON pages BEGIN
                INSERT INTO pages_fts(pages_fts, rowid, url, title, content_text) 
                VALUES('delete', old.rowid, old.url, old.title, old.content_text);
            END",
            [],
        )?;
        
        connection.execute(
            "CREATE TRIGGER IF NOT EXISTS pages_au AFTER UPDATE ON pages BEGIN
                INSERT INTO pages_fts(pages_fts, rowid, url, title, content_text) 
                VALUES('delete', old.rowid, old.url, old.title, old.content_text);
                INSERT INTO pages_fts(rowid, url, title, content_text) 
                VALUES (new.rowid, new.url, new.title, new.content_text);
            END",
            [],
        )?;
        
        Ok(())
    }
    
    /// Store a page in the database
    pub fn store_page(&self, page: PageData) -> Result<()> {
        // Get connection lock
        let conn = self.connection.lock().unwrap();
        
        // Check if the URL already exists
        let url_exists: bool = conn.query_row(
            "SELECT 1 FROM pages WHERE url = ?1 LIMIT 1",
            params![&page.url],
            |_| Ok(true)
        ).unwrap_or(false);
        
        if url_exists {
            // Update existing record
            debug!("Updating existing page: {}", page.url);
            
            let canonical_url = page.canonical_url.as_ref().map(|s| s.as_str());
            let meta_tags_json = serde_json::to_string(&page.meta_tags)?;
            let scraped_at = page.scraped_at.to_rfc3339();
            
            conn.execute(
                "UPDATE pages SET
                    language = ?1,
                    title = ?2,
                    meta_tags = ?3,
                    canonical_url = ?4,
                    content_text = ?5,
                    scraped_at = ?6
                WHERE url = ?7",
                params![
                    &page.language,
                    &page.title,
                    &meta_tags_json,
                    canonical_url,
                    &page.content_text,
                    &scraped_at,
                    &page.url
                ],
            )?;
        } else {
            // Insert new record
            debug!("Inserting new page: {}", page.url);
            
            let canonical_url = page.canonical_url.as_ref().map(|s| s.as_str());
            let meta_tags_json = serde_json::to_string(&page.meta_tags)?;
            let scraped_at = page.scraped_at.to_rfc3339();
            
            conn.execute(
                "INSERT INTO pages (
                    url, language, title, meta_tags, canonical_url, content_text, scraped_at
                ) VALUES (
                    ?1, ?2, ?3, ?4, ?5, ?6, ?7
                )",
                params![
                    &page.url,
                    &page.language,
                    &page.title,
                    &meta_tags_json,
                    canonical_url,
                    &page.content_text,
                    &scraped_at
                ],
            )?;
        }
        
        Ok(())
    }
    
    /// Search for pages matching a query
    pub fn search(&self, query: &str) -> Result<Vec<PageData>> {
        let conn = self.connection.lock().unwrap();
        
        let mut stmt = conn.prepare(
            "SELECT p.url, p.language, p.title, p.meta_tags, p.canonical_url, p.content_text, p.scraped_at
            FROM pages p
            JOIN pages_fts f ON p.rowid = f.rowid
            WHERE pages_fts MATCH ?1
            ORDER BY rank"
        )?;
        
        let search_query = format!("{}*", query);
        
        let page_iter = stmt.query_map(params![search_query], |row| {
            let url: String = row.get(0)?;
            let language: String = row.get(1)?;
            let title: String = row.get(2)?;
            let meta_tags_json: String = row.get(3)?;
            let canonical_url: Option<String> = row.get(4)?;
            let content_text: String = row.get(5)?;
            let scraped_at_str: String = row.get(6)?;
            
            let meta_tags = serde_json::from_str(&meta_tags_json)
                .unwrap_or_default();
                
            let scraped_at = chrono::DateTime::parse_from_rfc3339(&scraped_at_str)
                .unwrap_or_else(|_| chrono::Utc::now().into())
                .with_timezone(&chrono::Utc);
            
            Ok(PageData {
                url,
                language,
                title,
                meta_tags,
                canonical_url,
                content_text,
                scraped_at,
            })
        })?;
        
        let mut results = Vec::new();
        for page_result in page_iter {
            match page_result {
                Ok(page) => results.push(page),
                Err(e) => warn!("Error processing search result: {}", e),
            }
        }
        
        Ok(results)
    }
    
    /// Get all pages in the database
    pub fn get_all_pages(&self) -> Result<Vec<PageData>> {
        let conn = self.connection.lock().unwrap();
        
        let mut stmt = conn.prepare(
            "SELECT url, language, title, meta_tags, canonical_url, content_text, scraped_at
            FROM pages"
        )?;
        
        let page_iter = stmt.query_map([], |row| {
            let url: String = row.get(0)?;
            let language: String = row.get(1)?;
            let title: String = row.get(2)?;
            let meta_tags_json: String = row.get(3)?;
            let canonical_url: Option<String> = row.get(4)?;
            let content_text: String = row.get(5)?;
            let scraped_at_str: String = row.get(6)?;
            
            let meta_tags = serde_json::from_str(&meta_tags_json)
                .unwrap_or_default();
                
            let scraped_at = chrono::DateTime::parse_from_rfc3339(&scraped_at_str)
                .unwrap_or_else(|_| chrono::Utc::now().into())
                .with_timezone(&chrono::Utc);
            
            Ok(PageData {
                url,
                language,
                title,
                meta_tags,
                canonical_url,
                content_text,
                scraped_at,
            })
        })?;
        
        let mut results = Vec::new();
        for page_result in page_iter {
            match page_result {
                Ok(page) => results.push(page),
                Err(e) => warn!("Error processing page result: {}", e),
            }
        }
        
        Ok(results)
    }
    
    /// Get a page by URL
    pub fn get_page_by_url(&self, url: &str) -> Result<Option<PageData>> {
        let conn = self.connection.lock().unwrap();
        
        let result = conn.query_row(
            "SELECT url, language, title, meta_tags, canonical_url, content_text, scraped_at
            FROM pages
            WHERE url = ?1",
            params![url],
            |row| {
                let url: String = row.get(0)?;
                let language: String = row.get(1)?;
                let title: String = row.get(2)?;
                let meta_tags_json: String = row.get(3)?;
                let canonical_url: Option<String> = row.get(4)?;
                let content_text: String = row.get(5)?;
                let scraped_at_str: String = row.get(6)?;
                
                let meta_tags = serde_json::from_str(&meta_tags_json)
                    .unwrap_or_default();
                    
                let scraped_at = chrono::DateTime::parse_from_rfc3339(&scraped_at_str)
                    .unwrap_or_else(|_| chrono::Utc::now().into())
                    .with_timezone(&chrono::Utc);
                
                Ok(PageData {
                    url,
                    language,
                    title,
                    meta_tags,
                    canonical_url,
                    content_text,
                    scraped_at,
                })
            }
        );
        
        match result {
            Ok(page) => Ok(Some(page)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }
    
    /// Export pages to a JSON file
    pub fn export_to_json(&self, path: &Path) -> Result<()> {
        let pages = self.get_all_pages()?;
        let json = serde_json::to_string_pretty(&pages)?;
        fs::write(path, json)?;
        Ok(())
    }
}