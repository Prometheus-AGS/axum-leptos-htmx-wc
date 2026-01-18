#![allow(clippy::pedantic)]

use super::CertificationResult;

pub async fn execute_ui_test(
    test_id: &str,
    environment_id: &str,
) -> Result<CertificationResult, Box<dyn std::error::Error + Send + Sync>> {
    Ok(super::skipped_result(test_id, environment_id, "UI"))
}
