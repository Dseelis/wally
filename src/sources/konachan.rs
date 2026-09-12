use anyhow::{Context, Result};
use reqwest::Client;
use serde::Deserialize;

use crate::{
    cli::{Rating, Resolution, WallyConfig},
    history,
    sources::{Thumbnails, Wallpaper},
};


const API_URL: &str =
    "https://konachan.com/post.json";


#[derive(Debug, Deserialize)]
struct KonachanPost {
    id: u64,
    rating: String,
    width: u32,
    height: u32,
    file_url: Option<String>,
    preview_url: Option<String>,
    sample_url: Option<String>,
    jpeg_url: Option<String>,
}


pub async fn search(
    config: &WallyConfig,
) -> Result<Vec<Wallpaper>> {
    let client =
        Client::new();


    let requested_amount =
        config.amount as usize;


    let mut collected =
        Vec::with_capacity(
            requested_amount
        );


    let mut page =
        1u32;


    let mut skipped_from_history =
        0usize;


    println!();
    println!(
        "🔎 Searching Konachan..."
    );


    while collected.len()
        < requested_amount
    {
        let posts =
            search_page(
                &client,
                config,
                page,
            )
            .await?;


        if posts.is_empty() {
            break;
        }


        for post in posts {
            if !matches_resolution(
                &post,
                &config.resolution,
            ) {
                continue;
            }


            let wallpaper =
                match convert_post(post) {
                    Ok(value) => value,
                    Err(_) => continue,
                };


            if history::contains(
                &wallpaper
            ) {
                skipped_from_history += 1;
                continue;
            }


            collected.push(
                wallpaper
            );


            if collected.len()
                >= requested_amount
            {
                break;
            }
        }


        page += 1;


        if page > 100 {
            break;
        }
    }


    println!();

    println!(
        "✓ Found {} new wallpapers",
        collected.len()
    );


    if skipped_from_history > 0 {
        println!(
            "↳ Skipped {} already downloaded",
            skipped_from_history
        );
    }


    if collected.is_empty() {
        anyhow::bail!(
            "No new wallpapers found on Konachan.\n\
             All matching wallpapers may already be in history."
        );
    }


    Ok(collected)
}


async fn search_page(
    client: &Client,
    config: &WallyConfig,
    page: u32,
) -> Result<Vec<KonachanPost>> {
    let mut tags =
        Vec::<String>::new();


    if !config.tags
        .trim()
        .is_empty()
    {
        for tag in
            config.tags.split(',')
        {
            let tag =
                tag.trim();


            if tag.is_empty() {
                continue;
            }


            tags.push(
                tag.replace(
                    ' ',
                    "_",
                )
            );
        }
    }


    match config.rating {
        Rating::Sfw => {
            tags.push(
                "rating:s".to_string()
            );
        }


        Rating::Sketchy => {
            tags.push(
                "rating:s".to_string()
            );

            tags.push(
                "rating:q".to_string()
            );
        }


        Rating::Nsfw => {
            tags.push(
                "rating:e".to_string()
            );
        }
    }


    match &config.resolution {
        Resolution::Exact(
            width,
            height,
        ) => {
            tags.push(
                format!(
                    "width:{width}"
                )
            );

            tags.push(
                format!(
                    "height:{height}"
                )
            );
        }


        Resolution::Minimum(
            width,
            height,
        ) => {
            tags.push(
                format!(
                    "width:>={width}"
                )
            );

            tags.push(
                format!(
                    "height:>={height}"
                )
            );
        }


        Resolution::Ratio(_, _) => {}
    }


    let tag_query =
        tags.join(" ");


    let page_string =
        page.to_string();


    let response =
        client
            .get(API_URL)
            .query(&[
                (
                    "limit",
                    "100",
                ),
                (
                    "page",
                    &page_string,
                ),
                (
                    "tags",
                    &tag_query,
                ),
            ])
            .send()
            .await
            .context(
                "Failed to connect to Konachan",
            )?;


    let status =
        response.status();


    if !status.is_success() {
        let body =
            response
                .text()
                .await
                .unwrap_or_default();


        anyhow::bail!(
            "Konachan API returned HTTP {}\n{}",
            status,
            body
        );
    }


    response
        .json::<Vec<KonachanPost>>()
        .await
        .context(
            "Failed to parse Konachan response",
        )
}


fn matches_resolution(
    post: &KonachanPost,
    resolution: &Resolution,
) -> bool {
    match resolution {
        Resolution::Exact(
            width,
            height,
        ) => {
            post.width == *width
                && post.height == *height
        }


        Resolution::Minimum(
            width,
            height,
        ) => {
            post.width >= *width
                && post.height >= *height
        }


        Resolution::Ratio(
            target_width,
            target_height,
        ) => {
            let post_ratio =
                post.width as f64
                    / post.height as f64;


            let target_ratio =
                *target_width as f64
                    / *target_height as f64;


            let difference =
                (post_ratio - target_ratio)
                    .abs();


            difference <= 0.01
        }
    }
}


fn convert_post(
    post: KonachanPost,
) -> Result<Wallpaper> {
    let path =
        post.file_url
            .or(post.jpeg_url)
            .context(
                "Konachan post has no downloadable file URL",
            )?;


    let preview =
        post.sample_url
            .or(post.preview_url)
            .unwrap_or_else(
                || path.clone()
            );


    let file_type =
        detect_file_type(&path);


    let id =
        post.id.to_string();


    Ok(
        Wallpaper {
            id:
                id.clone(),

            url:
                format!(
                    "https://konachan.com/post/show/{id}"
                ),

            purity:
                post.rating,

            category:
                "anime".to_string(),

            dimension_x:
                post.width,

            dimension_y:
                post.height,

            resolution:
                format!(
                    "{}x{}",
                    post.width,
                    post.height
                ),

            path,

            file_type,

            thumbs:
                Thumbnails {
                    large:
                        preview.clone(),

                    original:
                        preview.clone(),

                    small:
                        preview,
                },
        }
    )
}


fn detect_file_type(
    path: &str,
) -> String {
    let path =
        path
            .split('?')
            .next()
            .unwrap_or(path);


    if path.ends_with(".png") {
        return "image/png".to_string();
    }


    if path.ends_with(".webp") {
        return "image/webp".to_string();
    }


    if path.ends_with(".gif") {
        return "image/gif".to_string();
    }


    "image/jpeg".to_string()
}