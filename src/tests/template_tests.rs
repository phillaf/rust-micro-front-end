#[cfg(test)]
mod tests {
    use crate::template::TemplateService;

    use std::fs;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_template_service_creation() {
        // We need to create a temporary template directory for testing
        let dir = tempdir().unwrap();
        let template_dir = dir.path().join("templates");
        fs::create_dir_all(&template_dir).unwrap();

        // Create a simple test template
        let test_template = template_dir.join("test.html");
        let mut file = fs::File::create(test_template).unwrap();
        writeln!(file, "<html><body>Hello, {{ name }}!</body></html>").unwrap();

        // Temporarily override template path (this requires environment manipulation which might not work well in tests)
        // Just test the initialization for now
        let service_result = TemplateService::new(false, false);
        assert!(service_result.is_ok());
    }

    // Note: In a real environment, we'd need to set up the template directory correctly
    // For now, this test just checks that the service can be instantiated
    // Additional tests would verify rendering templates with different caching settings
}
