use std::collections::HashMap;

#[derive(Debug, serde::Deserialize)]
struct Config {
    postcode: String,
}

#[derive(Debug, serde::Deserialize)]
enum Intensity {
    Low,
    Moderate,
    High,
}

#[derive(Debug, serde::Deserialize)]
struct RegionalResponse {
    forecast: u64,
    index: Intensity,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    pretty_env_logger::init();
    let config = Config {
        postcode: "E1W".to_string(),
    };
    let url = format!(
        "https://api.carbonintensity.org.uk/regional/postcode/{}",
        &config.postcode
    );
    log::trace!("{}", url);
    let resp = reqwest::get(&url).await?.text().await?;
    log::info!("{:#?}", resp);
    Ok(())
}
