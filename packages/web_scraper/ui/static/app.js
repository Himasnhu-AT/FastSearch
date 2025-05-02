// DOM elements
const tabButtons = document.querySelectorAll('.tab-button');
const tabPanes = document.querySelectorAll('.tab-pane');
const statsContent = document.getElementById('stats-content');
const pagesContent = document.getElementById('pages-content');
const searchInput = document.getElementById('search-input');
const searchButton = document.getElementById('search-button');
const modal = document.getElementById('page-modal');
const modalTitle = document.getElementById('modal-title');
const modalContent = document.getElementById('modal-content');
const closeModal = document.querySelector('.close');

// API endpoints
const API = {
    STATS: '/api/stats',
    PAGES: '/api/pages',
    PAGE: (url) => `/api/page/${encodeURIComponent(url)}`,
    SEARCH: (query) => `/api/search?q=${encodeURIComponent(query)}`
};

// Initialize the dashboard
document.addEventListener('DOMContentLoaded', () => {
    // Set up tab switching
    tabButtons.forEach(button => {
        button.addEventListener('click', () => {
            const tabId = button.getAttribute('data-tab');
            
            // Update active state for buttons
            tabButtons.forEach(btn => btn.classList.remove('active'));
            button.classList.add('active');
            
            // Update active state for panes
            tabPanes.forEach(pane => pane.classList.remove('active'));
            document.getElementById(tabId).classList.add('active');
            
            // Load content for the active tab
            if (tabId === 'stats') {
                loadStats();
            } else if (tabId === 'pages') {
                loadPages();
            }
        });
    });
    
    // Set up search
    searchButton.addEventListener('click', performSearch);
    searchInput.addEventListener('keypress', (e) => {
        if (e.key === 'Enter') {
            performSearch();
        }
    });
    
    // Set up modal close
    closeModal.addEventListener('click', () => {
        modal.style.display = 'none';
    });
    
    window.addEventListener('click', (e) => {
        if (e.target === modal) {
            modal.style.display = 'none';
        }
    });
    
    // Initial load
    loadStats();
});

// Load statistics
async function loadStats() {
    try {
        showLoader(statsContent);
        const response = await fetch(API.STATS);
        const data = await response.json();
        
        // Format and display stats
        statsContent.innerHTML = `<pre>${data.stats}</pre>`;
        
        // Refresh stats every 10 seconds
        setTimeout(loadStats, 10000);
    } catch (error) {
        statsContent.innerHTML = `<p class="error">Error loading statistics: ${error.message}</p>`;
    }
}

// Load all pages
async function loadPages() {
    try {
        showLoader(pagesContent);
        const response = await fetch(API.PAGES);
        const pages = await response.json();
        
        if (pages.length === 0) {
            pagesContent.innerHTML = '<p>No pages found in the database.</p>';
            return;
        }
        
        displayPages(pages);
    } catch (error) {
        pagesContent.innerHTML = `<p class="error">Error loading pages: ${error.message}</p>`;
    }
}

// Perform search
async function performSearch() {
    const query = searchInput.value.trim();
    if (!query) return;
    
    try {
        // Switch to pages tab
        tabButtons.forEach(btn => btn.classList.remove('active'));
        tabButtons[1].classList.add('active');
        
        tabPanes.forEach(pane => pane.classList.remove('active'));
        tabPanes[1].classList.add('active');
        
        showLoader(pagesContent);
        const response = await fetch(API.SEARCH(query));
        const pages = await response.json();
        
        if (pages.length === 0) {
            pagesContent.innerHTML = '<p>No pages found matching your search.</p>';
            return;
        }
        
        displayPages(pages);
    } catch (error) {
        pagesContent.innerHTML = `<p class="error">Error searching pages: ${error.message}</p>`;
    }
}

// Display pages in a grid
function displayPages(pages) {
    let html = '';
    
    pages.forEach(page => {
        html += `
            <div class="page-card" data-url="${page.url}">
                <h3>${escapeHtml(page.title)}</h3>
                <div class="url">${escapeHtml(page.url)}</div>
                <div class="description">${escapeHtml(page.description)}</div>
                <div class="meta">
                    <span>Language: ${page.language}</span>
                    <span>Scraped: ${formatDate(page.scraped_at)}</span>
                </div>
            </div>
        `;
    });
    
    pagesContent.innerHTML = html;
    
    // Add click event to page cards
    document.querySelectorAll('.page-card').forEach(card => {
        card.addEventListener('click', () => {
            const url = card.getAttribute('data-url');
            showPageDetails(url);
        });
    });
}

// Show page details in modal
async function showPageDetails(url) {
    try {
        modalTitle.innerText = 'Loading...';
        modalContent.innerHTML = '<div class="loader"></div>';
        modal.style.display = 'block';
        
        const response = await fetch(API.PAGE(url));
        const page = await response.json();
        
        if (page.error) {
            modalTitle.innerText = 'Error';
            modalContent.innerHTML = `<p class="error">${page.error}</p>`;
            return;
        }
        
        modalTitle.innerText = page.title;
        
        let metaTags = '';
        if (page.meta_tags && page.meta_tags.length > 0) {
            metaTags = '<h3>Meta Tags</h3><ul>';
            page.meta_tags.forEach(tag => {
                metaTags += `<li><strong>${escapeHtml(tag.name)}:</strong> ${escapeHtml(tag.content)}</li>`;
            });
            metaTags += '</ul>';
        }
        
        modalContent.innerHTML = `
            <p><strong>URL:</strong> <a href="${page.url}" target="_blank">${escapeHtml(page.url)}</a></p>
            <p><strong>Language:</strong> ${page.language}</p>
            <p><strong>Scraped at:</strong> ${formatDate(page.scraped_at)}</p>
            ${page.canonical_url ? `<p><strong>Canonical URL:</strong> ${escapeHtml(page.canonical_url)}</p>` : ''}
            ${metaTags}
            <h3>Content</h3>
            <div class="content-preview">${escapeHtml(page.content_text)}</div>
        `;
    } catch (error) {
        modalTitle.innerText = 'Error';
        modalContent.innerHTML = `<p class="error">Failed to load page details: ${error.message}</p>`;
    }
}

// Helper functions
function showLoader(element) {
    element.innerHTML = '<div class="loader"></div>';
}

function formatDate(dateString) {
    const date = new Date(dateString);
    return date.toLocaleString();
}

function escapeHtml(unsafe) {
    return unsafe
        .replace(/&/g, "&amp;")
        .replace(/</g, "&lt;")
        .replace(/>/g, "&gt;")
        .replace(/"/g, "&quot;")
        .replace(/'/g, "&#039;");
}