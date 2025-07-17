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
    minify_enabled: bool,
}

impl TemplateService {
    /// Create a new template service with caching and minification
    pub fn new(cache_enabled: bool, minify_enabled: bool) -> Result<Self> {
        let mut env = Environment::new();
        env.set_loader(minijinja::path_loader("templates"));
        
        info!("Template service initialized with caching: {}, minification: {}", cache_enabled, minify_enabled);
        
        Ok(Self {
            environment: Arc::new(RwLock::new(env)),
            cache_enabled,
            template_cache: Arc::new(RwLock::new(HashMap::new())),
            minify_enabled,
        })
    }
    
    /// Render a template with the given context
    pub fn render(&self, template_name: &str, context: Value) -> Result<String> {
        // For user-specific templates, always render fresh to avoid stale data
        let should_cache = self.cache_enabled && !self.is_user_specific_template(template_name);
        
        if should_cache {
            let cache_key = format!("{}:{}:{}", template_name, self.context_hash(&context), self.minify_enabled);
            
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
            
            // Apply minification if enabled
            let final_html = if self.minify_enabled {
                self.minify_html(&html)?
            } else {
                html
            };
            
            // Cache the result
            self.cache_html(cache_key, final_html.clone());
            debug!("Template cached for: {}", template_name);
            
            Ok(final_html)
        } else {
            // Don't use cache - render fresh every time
            debug!("Rendering fresh template (no cache): {}", template_name);
            
            let env = self.environment.read()
                .map_err(|_| anyhow::anyhow!("Failed to acquire template environment lock"))?;
            
            let template = env.get_template(template_name)?;
            let html = template.render(context)?;
            
            // Apply minification if enabled
            let final_html = if self.minify_enabled {
                self.minify_html(&html)?
            } else {
                html
            };
            
            Ok(final_html)
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
    
    /// Minify HTML content for production performance
    fn minify_html(&self, html: &str) -> Result<String> {
        use minify_html::{minify, Cfg};
        
        let cfg = Cfg {
            do_not_minify_doctype: true,
            ensure_spec_compliant_unquoted_attribute_values: true,
            keep_closing_tags: true,
            keep_html_and_head_opening_tags: true,
            keep_spaces_between_attributes: true, // Keep spaces for better compatibility
            keep_comments: false,
            keep_input_type_text_attr: true, // Keep input attributes
            keep_ssi_comments: false,
            preserve_brace_template_syntax: false,
            preserve_chevron_percent_template_syntax: false,
            minify_css: false, // Disable CSS minification for now
            minify_js: false,  // Disable JS minification for now
            remove_bangs: false,
            remove_processing_instructions: false,
        };
        
        let minified = minify(html.as_bytes(), &cfg);
        String::from_utf8(minified)
            .map_err(|e| anyhow::anyhow!("Failed to convert minified HTML to string: {}", e))
    }
    
    /// Health check for template service
    pub fn health_check(&self) -> bool {
        // Check if template environment is accessible
        let env_accessible = self.environment.read().is_ok();
        
        // Check if we can access the cache
        let cache_accessible = self.template_cache.read().is_ok();
        
        // Check if base template exists and can be loaded
        let template_accessible = if env_accessible {
            if let Ok(env) = self.environment.read() {
                env.get_template("base.html").is_ok()
            } else {
                false
            }
        } else {
            false
        };
        
        // All checks must pass
        env_accessible && cache_accessible && template_accessible
    }
}

/// Create template service based on environment
pub fn create_template_service() -> Result<TemplateService> {
    let cache_enabled = std::env::var("TEMPLATE_CACHE_ENABLED")
        .unwrap_or_else(|_| "true".to_string())
        .parse()
        .unwrap_or(true);
    
    let minify_enabled = std::env::var("MINIFY_ENABLED")
        .unwrap_or_else(|_| "false".to_string())
        .parse()
        .unwrap_or(false);
    
    TemplateService::new(cache_enabled, minify_enabled)
}
