use axum::{
    http::{header, StatusCode},
    response::{IntoResponse, Response},
};

/// GET /manifest.json - Web App Manifest for PWA support
pub async fn get_manifest() -> impl IntoResponse {
    let manifest = serde_json::json!({
        "name": "Micro Frontend App",
        "short_name": "MicroApp",
        "description": "A high-performance, containerized micro web-application demonstrating modern web development constraints with Rust, Docker, and micro front-end architecture",
        "start_url": "/",
        "display": "standalone",
        "background_color": "#f5f5f5",
        "theme_color": "#3498db",
        "icons": [
            {
                "src": "data:image/svg+xml,<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 100 100'><text y='.9em' font-size='90'>⚡</text></svg>",
                "sizes": "any",
                "type": "image/svg+xml"
            }
        ],
        "categories": ["business", "productivity"],
        "lang": "en-US",
        "orientation": "portrait-primary"
    });

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::CACHE_CONTROL, "public, max-age=3600")
        .body(manifest.to_string())
        .unwrap()
}

/// GET /robots.txt - Robots.txt for SEO
pub async fn get_robots_txt() -> impl IntoResponse {
    let robots = "User-agent: *\nAllow: /\n\nSitemap: https://example.com/sitemap.xml";
    
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/plain")
        .header(header::CACHE_CONTROL, "public, max-age=86400")
        .body(robots.to_string())
        .unwrap()
}

/// GET /sitemap.xml - Basic sitemap for SEO
pub async fn get_sitemap() -> impl IntoResponse {
    let sitemap = r#"<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
    <url>
        <loc>https://example.com/</loc>
        <changefreq>monthly</changefreq>
        <priority>1.0</priority>
    </url>
    <url>
        <loc>https://example.com/edit</loc>
        <changefreq>weekly</changefreq>
        <priority>0.8</priority>
    </url>
    <url>
        <loc>https://example.com/display/username/exampleuser</loc>
        <changefreq>daily</changefreq>
        <priority>0.9</priority>
    </url>
</urlset>"#;
    
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/xml")
        .header(header::CACHE_CONTROL, "public, max-age=86400")
        .body(sitemap.to_string())
        .unwrap()
}
