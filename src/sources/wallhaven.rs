use std::time::Duration;

use anyhow::{Context, Result};
use reqwest::Client;
use serde::Deserialize;
use tokio::time::sleep;

use crate::{
    cli::{Rating, Resolution, WallyConfig},
    history,
    sources::{Thumbnails, Wallpaper},
};

const API_URL: &str = "https://wallhaven.cc/api/v1/search";
const MAX_RETRIES: u32 = 5;

#[derive(Debug, Deserialize)]
pub struct WallhavenResponse {
    pub data: Vec<Wallpaper>,
    pub meta: Meta,
}

#[derive(Debug, Deserialize)]
pub struct Meta {
    pub current_page: u32,
    pub last_page: u32,
    pub total: u32,
}

#[derive(Debug, Deserialize)]
struct WallhavenApiResponse {
    data: Vec<WallhavenWallpaper>,
    meta: Meta,
}

#[derive(Debug, Deserialize)]
struct WallhavenWallpaper {
    id: String,
    url: String,
    purity: String,
    category: String,
    dimension_x: u32,
    dimension_y: u32,
    resolution: String,
    path: String,
    file_type: String,
    thumbs: Thumbnails,
}

/// Search wallpapers on Wallhaven.
pub async fn search(config: &WallyConfig) -> Result<WallhavenResponse> {
    let client = Client::builder()
        .user_agent("Wally/0.1.0")
        .build()
        .context("Failed to create HTTP client")?;

    let requested_amount = config.amount as usize;

    let mut collected = Vec::with_capacity(requested_amount);
    let mut current_page = 1u32;
    let mut last_page = 1u32;
    let mut skipped_from_history = 0usize;

    println!();
    println!("🔎 Searching Wallhaven...");

    while collected.len() < requested_amount {
        let response = search_page(&client, config, current_page).await?;

        last_page = response.meta.last_page;

        for wallpaper in response.data {
            let wallpaper = convert_wallpaper(wallpaper);

            // Don't show wallpapers that have already been downloaded.
            if history::contains(&wallpaper) {
                skipped_from_history += 1;
                continue;
            }

            collected.push(wallpaper);

            if collected.len() >= requested_amount {
                break;
            }
        }

        if current_page >= last_page {
            break;
        }

        current_page += 1;
    }

    println!();
    println!("✓ Found {} new wallpapers", collected.len());

    if skipped_from_history > 0 {
        println!(
            "↳ Skipped {} already downloaded",
            skipped_from_history
        );
    }

    if collected.is_empty() {
        anyhow::bail!(
            "No new wallpapers found.\n\
             All matching wallpapers may already be in history."
        );
    }

    let collected_len = collected.len() as u32;

    Ok(WallhavenResponse {
        data: collected,
        meta: Meta {
            current_page,
            last_page,
            total: collected_len,
        },
    })
}

/// Request one Wallhaven API page with retry handling.
async fn search_page(
    client: &Client,
    config: &WallyConfig,
    page: u32,
) -> Result<WallhavenApiResponse> {
    let mut params = Vec::new();

    // Tags
    if !config.tags.trim().is_empty() {
        let tags = config
            .tags
            .split(',')
            .map(str::trim)
            .filter(|tag| !tag.is_empty())
            .collect::<Vec<_>>()
            .join("+");

        params.push(("q", tags));
    }

    // Rating
    let purity = match config.rating {
        Rating::Sfw => "100",
        Rating::Sketchy => "110",
        Rating::Nsfw => "111",
    };

    params.push(("purity", purity.to_string()));

    // Categories:
    // 100 = General
    // 010 = Anime
    // 001 = People
    // 111 = all
    params.push(("categories", "111".to_string()));

    // Resolution
    match &config.resolution {
        Resolution::Exact(width, height) => {
            params.push((
                "resolutions",
                format!("{width}x{height}"),
            ));
        }

        Resolution::Minimum(width, height) => {
            params.push((
                "atleast",
                format!("{width}x{height}"),
            ));
        }

        Resolution::Ratio(width, height) => {
            params.push((
                "ratios",
                format!("{width}x{height}"),
            ));
        }
    }

    // Random sorting.
    params.push(("sorting", "random".to_string()));

    params.push(("page", page.to_string()));

    // Optional Wallhaven API key.
    if let Ok(key) = std::env::var("WALLHAVEN_API_KEY") {
        params.push(("apikey", key));
    }

    let mut attempt = 0u32;

    loop {
        attempt += 1;

        let response = client
            .get(API_URL)
            .query(&params)
            .send()
            .await;

        let response = match response {
            Ok(response) => response,

            Err(error) => {
                if attempt >= MAX_RETRIES {
                    return Err(anyhow::anyhow!(
                        "Failed to connect to Wallhaven after {} attempts: {}",
                        MAX_RETRIES,
                        error
                    ));
                }

                let delay = retry_delay(attempt);

                println!(
                    "⚠ Wallhaven connection failed. \
                     Retrying in {}s... ({}/{})",
                    delay.as_secs(),
                    attempt,
                    MAX_RETRIES
                );

                sleep(delay).await;
                continue;
            }
        };

        let status = response.status();

        // Temporary server/gateway errors.
        if status == reqwest::StatusCode::BAD_GATEWAY
            || status == reqwest::StatusCode::SERVICE_UNAVAILABLE
            || status == reqwest::StatusCode::GATEWAY_TIMEOUT
        {
            if attempt >= MAX_RETRIES {
                let body = response
                    .text()
                    .await
                    .unwrap_or_default();

                anyhow::bail!(
                    "Wallhaven API returned HTTP {} \
                     after {} attempts\n{}",
                    status,
                    MAX_RETRIES,
                    body
                );
            }

            let delay = retry_delay(attempt);

            println!(
                "⚠ Wallhaven returned HTTP {}. \
                 Retrying in {}s... ({}/{})",
                status,
                delay.as_secs(),
                attempt,
                MAX_RETRIES
            );

            sleep(delay).await;
            continue;
        }

        // Other HTTP errors are not considered temporary.
        if !status.is_success() {
            let body = response
                .text()
                .await
                .unwrap_or_default();

            anyhow::bail!(
                "Wallhaven API returned HTTP {}\n{}",
                status,
                body
            );
        }

        return response
            .json::<WallhavenApiResponse>()
            .await
            .context("Failed to parse Wallhaven response");
    }
}

/// Exponential retry delay:
///
/// attempt 1 -> 1s
/// attempt 2 -> 2s
/// attempt 3 -> 4s
/// attempt 4 -> 8s
/// attempt 5 -> 8s
fn retry_delay(attempt: u32) -> Duration {
    let seconds = 2u64
        .pow(attempt.saturating_sub(1))
        .min(8);

    Duration::from_secs(seconds)
}

/// Convert Wallhaven's API structure into Wally's common structure.
fn convert_wallpaper(
    wallpaper: WallhavenWallpaper,
) -> Wallpaper {
    Wallpaper {
        id: wallpaper.id,
        url: wallpaper.url,
        purity: wallpaper.purity,
        category: wallpaper.category,
        dimension_x: wallpaper.dimension_x,
        dimension_y: wallpaper.dimension_y,
        resolution: wallpaper.resolution,
        path: wallpaper.path,
        file_type: wallpaper.file_type,
        thumbs: wallpaper.thumbs,
    }
}