#[cfg(test)]
mod tests {
    use crate::database::mock::MockUserDatabase;
    use crate::handlers::get_api_username::get_api_username;
    use crate::handlers::get_display::get_display_username;
    use crate::metrics::AppMetrics;
    use crate::router::AppState;
    use crate::template::TemplateService;
    use crate::validation::ValidatedUsername;

    use axum::extract::{Path, State};
    use std::sync::Arc;
    use std::time::{Duration, Instant};

    /// Run a simple benchmark for handler functions
    async fn run_benchmark<F, Fut>(name: &str, iterations: usize, setup: fn() -> F, run: fn(F) -> Fut) -> Duration
    where
        Fut: std::future::Future<Output = ()>,
    {
        // Warm-up phase
        let setup_val = setup();
        run(setup_val).await;

        // Benchmark phase
        let mut total_duration = Duration::new(0, 0);

        for _ in 0..iterations {
            let setup_val = setup();
            let start = Instant::now();
            run(setup_val).await;
            total_duration += start.elapsed();
        }

        let avg_duration = total_duration / iterations as u32;
        println!("Benchmark [{}] - Avg over {} iterations: {:?}", name, iterations, avg_duration);

        total_duration
    }

    #[tokio::test]
    async fn benchmark_display_handler() {
        let iterations = 100;

        let setup = || {
            // Create dependencies for handler
            let db = Arc::new(MockUserDatabase::new());
            let template_service = TemplateService::new(true, true).unwrap(); // Enable caching and minification

            let app_state = Arc::new(AppState {
                database: db,
                template_service,
                metrics: AppMetrics::new_for_tests(),
            });

            let state = State(app_state);
            let username = Path("admin".to_string());

            (state, username)
        };

        let run = |(state, username)| async move {
            let _ = get_display_username(state, username).await;
        };

        run_benchmark("display_handler", iterations, setup, run).await;
    }

    #[tokio::test]
    async fn benchmark_api_handler() {
        let iterations = 100;

        let setup = || {
            // Create dependencies for handler
            let db = Arc::new(MockUserDatabase::new());
            let template_service = TemplateService::new(true, true).unwrap();

            let app_state = Arc::new(AppState {
                database: db,
                template_service,
                metrics: AppMetrics::new_for_tests(),
            });

            let state = State(app_state);
            let username = Path("admin".to_string());

            (state, username)
        };

        let run = |(state, username)| async move {
            let _ = get_api_username(state, username).await;
        };

        run_benchmark("api_handler", iterations, setup, run).await;
    }

    #[tokio::test]
    async fn benchmark_username_validation() {
        let iterations = 1000; // More iterations for this lightweight operation

        let setup = || "validusername".to_string();

        let run = |username| async move {
            let _ = ValidatedUsername::new(username);
        };

        run_benchmark("username_validation", iterations, setup, run).await;
    }

    #[tokio::test]
    async fn benchmark_template_rendering() {
        let iterations = 50;

        let setup = || {
            let template_service = TemplateService::new(false, false).unwrap(); // No caching for baseline
            let context = minijinja::context! {
                username => "testuser",
                display_name => "Test User",
                title => "Test Page",
                description => "Test Description",
                keywords => "test, benchmark"
            };

            (template_service, context)
        };

        let run = |(template_service, context): (TemplateService, minijinja::value::Value)| async move {
            let _ = template_service.render("display.html", context);
        };

        let uncached_time = run_benchmark("template_rendering_uncached", iterations, setup, run).await;

        // Now test with caching enabled
        let setup_cached = || {
            let template_service = TemplateService::new(true, true).unwrap(); // With caching
            let context = minijinja::context! {
                username => "testuser",
                display_name => "Test User",
                title => "Test Page",
                description => "Test Description",
                keywords => "test, benchmark"
            };

            (template_service, context)
        };

        let cached_time = run_benchmark("template_rendering_cached", iterations, setup_cached, run).await;

        let speedup = if cached_time.as_nanos() > 0 {
            uncached_time.as_nanos() as f64 / cached_time.as_nanos() as f64
        } else {
            0.0
        };

        println!("Template caching provides {:.2}x speedup", speedup);
    }
}
