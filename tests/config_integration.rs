use axum_leptos_htmx_wc::config::AppConfig;
use serial_test::serial;
use std::env;
use std::fs;

// Helper to clear environment variables that might interfere with tests
// Helper to clear environment variables that might interfere with tests
fn setup_env_vars() {
    // SAFETY: Tests are serialized with `serial_test`, so process-wide env mutation is controlled.
    unsafe {
        env::set_var("UAR_TEST_MODE", "1");
        env::remove_var("UAR_SERVER__PORT");
        env::remove_var("UAR__SERVER__PORT");
        env::remove_var("UAR_SECURITY__JWT_REQUIRED");
        env::remove_var("CONFIG_FILE");

        // Set required fields to dummy values for testing
        // Note: config-rs with separator("__") expects UAR__SECURITY__JWT_SECRET
        env::set_var("UAR__SECURITY__JWT_SECRET", "test_secret_123");

        // Use double underscore for all fields that config-rs needs to map
        env::set_var("UAR__PERSISTENCE__PROVIDER", "postgres");
        env::set_var(
            "UAR__PERSISTENCE__DATABASE_URL",
            "postgres://user:pass@localhost:5432/db",
        );

        // Vector dimension relies on EnvSource
        env::set_var("UAR__PERSISTENCE__VECTOR_DIMENSION", "384");
    }
}

#[test]
#[serial]
fn test_default_config() {
    setup_env_vars();

    // reset args to avoid picking up test runner args
    // In a real integration scenario for CLI, we might subprocess, but here we test the load() logic
    // We can't easily mock Clap in-process without hacks, so we rely on env vars and file loading primarily here
    // or we'd need to restructure AppConfig::load to take args explicitly.
    // For now, let's test Env Var overrides which is the most critical logic to verify.

    let config = AppConfig::load_from_args(Vec::<String>::new());
    // It might fail if no config file and we are running in an environment where default paths don't exist
    // But defaults should kick in.
    assert!(config.is_ok());
    let config = config.unwrap();
    assert_eq!(config.server.port, 3000); // Default
}

#[test]
#[serial]
fn test_env_override() {
    setup_env_vars();
    // SAFETY: Tests are serialized with `serial_test`, so process-wide env mutation is controlled.
    unsafe {
        env::set_var("UAR__SERVER__PORT", "9090");
    }

    let config = AppConfig::load_from_args(Vec::<String>::new()).expect("Failed to load config");
    assert_eq!(config.server.port, 9090);

    setup_env_vars();
}

#[test]
#[serial]
fn test_file_load() {
    setup_env_vars();

    let config_content = r#"
server:
  port: 7070

security:
  jwt_secret: "test_secret_for_integration_testing_only_do_not_use_in_prod"

persistence:
  provider: "memory"
"#;

    let file_path = "test_config.yaml";
    fs::write(file_path, config_content).expect("Failed to write temp config");

    // Tell AppConfig to use this file via Env Var (mocking CLI arg indirectly)
    // SAFETY: Tests are serialized with `serial_test`, so process-wide env mutation is controlled.
    unsafe {
        env::set_var("CONFIG_FILE", file_path);
    }

    let config =
        AppConfig::load_from_args(Vec::<String>::new()).expect("Failed to load config from file");
    assert_eq!(config.server.port, 7070);

    fs::remove_file(file_path).unwrap();
    setup_env_vars();
}

#[test]
#[serial]
fn test_cwd_config_fallback() {
    setup_env_vars();

    // Create ./config.yaml
    let config_content = r#"
server:
  port: 6060

security:
  jwt_secret: "test_secret_for_integration_testing_only_do_not_use_in_prod"

persistence:
  provider: "memory"
"#;
    let cwd_path = "config.yaml";
    fs::write(cwd_path, config_content).expect("Failed to write ./config.yaml");

    // No Env var, No CLI (simulated)
    // Should pick up ./config.yaml
    let config = AppConfig::load_from_args(Vec::<String>::new()).expect("Failed to load config");

    // Clean up BEFORE assertion to ensure cleanup happens even if assert fails?
    // Ideally use a specialized fixture or try/catch, but for simple integration test:
    let result = std::panic::catch_unwind(|| {
        assert_eq!(config.server.port, 6060);
    });

    fs::remove_file(cwd_path).unwrap();

    if let Err(e) = result {
        std::panic::resume_unwind(e);
    }
}
