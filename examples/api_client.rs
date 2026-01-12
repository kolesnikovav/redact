//! Example API client demonstrating REST API usage
//!
//! First start the server with: cargo run --bin redact-api
//! Then run this client with: cargo run --example api_client

use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Redact API Client Example ===\n");

    let base_url = "http://localhost:8080";
    let client = reqwest::Client::new();

    // 1. Health check
    println!("--- Health Check ---");
    let health_response = client
        .get(&format!("{}/health", base_url))
        .send()
        .await?;

    println!("Status: {}", health_response.status());
    let health: serde_json::Value = health_response.json().await?;
    println!("{}\n", serde_json::to_string_pretty(&health)?);

    // 2. Analyze endpoint
    println!("--- Analyze Text ---");
    let analyze_request = json!({
        "text": "Contact John Doe at john@example.com or call (555) 123-4567. SSN: 123-45-6789",
        "language": "en"
    });

    let analyze_response = client
        .post(&format!("{}/api/v1/analyze", base_url))
        .json(&analyze_request)
        .send()
        .await?;

    println!("Status: {}", analyze_response.status());
    let analysis: serde_json::Value = analyze_response.json().await?;
    println!("{}\n", serde_json::to_string_pretty(&analysis)?);

    // 3. Anonymize endpoint
    println!("--- Anonymize Text ---");
    let anonymize_request = json!({
        "text": "Email john@example.com, SSN 123-45-6789",
        "language": "en",
        "config": {
            "strategy": "replace"
        }
    });

    let anonymize_response = client
        .post(&format!("{}/api/v1/anonymize", base_url))
        .json(&anonymize_request)
        .send()
        .await?;

    println!("Status: {}", anonymize_response.status());
    let anonymized: serde_json::Value = anonymize_response.json().await?;
    println!("{}\n", serde_json::to_string_pretty(&anonymized)?);

    // 4. Anonymize with mask strategy
    println!("--- Anonymize with Masking ---");
    let mask_request = json!({
        "text": "Credit card: 4532015112830366",
        "language": "en",
        "config": {
            "strategy": "mask",
            "mask_char": "*",
            "mask_start_chars": 4,
            "mask_end_chars": 4,
            "preserve_format": false
        }
    });

    let mask_response = client
        .post(&format!("{}/api/v1/anonymize", base_url))
        .json(&mask_request)
        .send()
        .await?;

    let masked: serde_json::Value = mask_response.json().await?;
    println!("{}\n", serde_json::to_string_pretty(&masked)?);

    Ok(())
}
