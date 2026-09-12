use std::path::PathBuf;

use anyhow::{Context, Result};
use image::DynamicImage;
use reqwest::Client;
use tokio::fs;
use tokio::io::AsyncWriteExt;

use crate::{
    history,
    sources::Wallpaper,
};


pub async fn load_previews(
    wallpapers: &[Wallpaper],
) -> Result<Vec<DynamicImage>> {
    let client =
        Client::new();


    let mut images =
        Vec::with_capacity(
            wallpapers.len()
        );


    println!();
    println!(
        "🖼 Loading previews..."
    );


    for (index, wallpaper)
        in wallpapers.iter().enumerate()
    {
        print!(
            "\rLoading preview {}/{}...",
            index + 1,
            wallpapers.len()
        );


        let response =
            client
                .get(
                    &wallpaper.thumbs.large
                )
                .send()
                .await
                .with_context(|| {
                    format!(
                        "Failed to download preview {}",
                        wallpaper.id
                    )
                })?;


        let bytes =
            response
                .bytes()
                .await
                .with_context(|| {
                    format!(
                        "Failed to read preview {}",
                        wallpaper.id
                    )
                })?;


        let image =
            image::load_from_memory(
                &bytes
            )
            .with_context(|| {
                format!(
                    "Failed to decode preview {}",
                    wallpaper.id
                )
            })?;


        images.push(image);
    }


    println!();
    println!(
        "✓ Previews loaded!"
    );


    Ok(images)
}


pub async fn download_wallpaper(
    wallpaper: &Wallpaper,
    directory: &str,
) -> Result<PathBuf> {
    let directory =
        expand_home(directory);


    fs::create_dir_all(
        &directory
    )
    .await
    .with_context(|| {
        format!(
            "Failed to create directory: {}",
            directory.display()
        )
    })?;


    let extension =
        extension_from_wallpaper(
            wallpaper
        );


    let filename =
        format!(
            "{}_{}.{}",
            wallpaper.id,
            wallpaper.dimension_x,
            extension
        );


    let path =
        directory.join(filename);


    /*
     * CACHE
     */

    if fs::try_exists(&path)
        .await
        .unwrap_or(false)
    {
        println!(
            "✓ Already downloaded: {}",
            path.display()
        );


        if !history::contains(
            wallpaper
        ) {
            history::add(
                wallpaper,
                &path,
            )?;
        }


        return Ok(path);
    }


    let client =
        Client::new();


    println!(
        "⬇ Downloading {}...",
        wallpaper.id
    );


    let response =
        client
            .get(&wallpaper.path)
            .send()
            .await
            .with_context(|| {
                format!(
                    "Failed to download wallpaper {}",
                    wallpaper.id
                )
            })?;


    if !response.status().is_success() {
        anyhow::bail!(
            "Failed to download {}: HTTP {}",
            wallpaper.id,
            response.status()
        );
    }


    let bytes =
        response
            .bytes()
            .await
            .with_context(|| {
                format!(
                    "Failed to read wallpaper {}",
                    wallpaper.id
                )
            })?;


    let temp_path =
        path.with_extension(
            format!(
                "{extension}.part"
            )
        );


    let mut file =
        fs::File::create(
            &temp_path
        )
        .await
        .with_context(|| {
            format!(
                "Failed to create temporary file {}",
                temp_path.display()
            )
        })?;


    file.write_all(&bytes)
        .await
        .with_context(|| {
            format!(
                "Failed to write temporary file {}",
                temp_path.display()
            )
        })?;


    file.sync_all()
        .await
        .with_context(|| {
            format!(
                "Failed to sync temporary file {}",
                temp_path.display()
            )
        })?;


    drop(file);


    fs::rename(
        &temp_path,
        &path,
    )
    .await
    .with_context(|| {
        format!(
            "Failed to finalize wallpaper {}",
            path.display()
        )
    })?;


    history::add(
        wallpaper,
        &path,
    )
    .context(
        "Wallpaper downloaded, but failed to save history"
    )?;


    println!(
        "✓ Saved: {}",
        path.display()
    );


    Ok(path)
}


fn extension_from_wallpaper(
    wallpaper: &Wallpaper,
) -> &str {
    match wallpaper.file_type.as_str() {
        "image/png" => "png",
        "image/webp" => "webp",
        "image/gif" => "gif",
        "image/jpeg" => "jpg",
        _ => "jpg",
    }
}


fn expand_home(
    path: &str,
) -> PathBuf {
    if path == "~" {
        return dirs::home_dir()
            .unwrap_or_else(
                || PathBuf::from(".")
            );
    }


    if let Some(rest) =
        path.strip_prefix("~/")
    {
        if let Some(home) =
            dirs::home_dir()
        {
            return home.join(rest);
        }
    }


    PathBuf::from(path)
}