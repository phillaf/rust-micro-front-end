use anyhow::Result;
use minijinja::{Environment, Value};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tracing::{debug, info};

/// Template cache service for improved performance
#[derive(Clone)]
pub struct TemplateService {
    environment: Arc<RwLock<Environment<'static>>>,
    cache_enabled: bool,
    template_cache: Arc<RwLock<HashMap<String, String>>>,
}

impl TemplateService {
    /// Create a new template service with caching
    pub fn new(cache_enabled: bool) -> Result<Self> {
        let mut env = Environment::new();
        env.set_loader(minijinja::path_loader("templates"));
        
        info!("Template service initialized with caching: {}", cache_enabled);
        
        Ok(Self {
            environment: Arc::new(RwLock::new(env)),
            cache_enabled,
            template_cache: Arc::new(RwLock::new(HashMap::new())),
        })
    }
    
    /// Render a template with the given context
    pub fn render(&self, template_name: &str, context: Value) -> Result<String> {
        // For user-specific templates, always render fresh to avoid stale data
        let should_cache = self.cache_enabled && !self.is_user_specific_template(template_name);
        
        if should_cache {
            let cache_key = format!("{}:{}", template_name, self.context_hash(&context));
            
            // Check cache first if enabled
            if let Some(cached_html) = self.get_cached_html(&cache_key) {
                debug!("Template cache hit for: {}", template_name);
                return Ok(cached_html);
            }
            
            // Render template
            let env = self.environment.read()
                .map_err(|_| anyhow::anyhow!("Failed to acquire template environment lock"))?;
            
            let template = env.get_template(template_name)?;
            let html = template.render(context)?;
            
            // Cache the result
            self.cache_html(cache_key, html.clone());
            debug!("Template cached for: {}", template_name);
            
            Ok(html)
        } else {
            // Don't use cache - render fresh every time
            debug!("Rendering fresh template (no cache): {}", template_name);
            
            let env = self.environment.read()
                .map_err(|_| anyhow::anyhow!("Failed to acquire template environment lock"))?;
            
            let template = env.get_template(template_name)?;
            let html = template.render(context)?;
            
            Ok(html)
        }
    }
    
    /// Clear template cache (useful for development)
    #[allow(dead_code)]
    pub fn clear_cache(&self) {
        if let Ok(mut cache) = self.template_cache.write() {
            cache.clear();
            info!("Template cache cleared");
        }
    }
    
    /// Get cached HTML if available
    fn get_cached_html(&self, key: &str) -> Option<String> {
        self.template_cache.read()
            .ok()
            .and_then(|cache| cache.get(key).cloned())
    }
    
    /// Cache rendered HTML
    fn cache_html(&self, key: String, html: String) {
        if let Ok(mut cache) = self.template_cache.write() {
            // Limit cache size to prevent memory issues
            if cache.len() >= 100 {
                cache.clear(); // Simple eviction strategy
            }
            cache.insert(key, html);
        }
    }
    
    /// Create a simple hash of the context for caching
    fn context_hash(&self, context: &Value) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        // Simple hash based on context string representation
        format!("{:?}", context).hash(&mut hasher);
        hasher.finish()
    }
    
    /// Check if template contains user-specific content that shouldn't be cached
    fn is_user_specific_template(&self, template_name: &str) -> bool {
        // Disable caching for all user-facing templates to ensure fresh data
        matches!(template_name, "display.html" | "edit.html")
    }
}

/// Create template service based on environment
pub fn create_template_service() -> Result<TemplateService> {
    let cache_enabled = std::env::var("TEMPLATE_CACHE_ENABLED")
        .unwrap_or_else(|_| "true".to_string())
        .parse()
        .unwrap_or(true);
    
    TemplateService::new(cache_enabled)
}
