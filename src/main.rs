mod cli;
mod downloader;
mod history;
mod locations;
mod sources;
mod ui;
mod wallpaper;

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(
    name = "wally",
    version,
    about = "Wallpaper downloader and manager"
)]
struct Args {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Start interactive wallpaper search
    Search,

    /// Download one random new wallpaper
    Random,

    /// Manage downloaded wallpaper history
    History {
        #[command(subcommand)]
        action: Option<HistoryAction>,
    },
}

#[derive(Debug, Subcommand)]
enum HistoryAction {
    /// Remove all history entries
    Clear,

    /// Remove one wallpaper from history
    Remove {
        /// Wallpaper ID
        id: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    match args.command {
        None | Some(Command::Search) => {
            run_search().await?;
        }

        Some(Command::Random) => {
            run_random().await?;
        }

        Some(Command::History { action }) => {
            run_history(action)?;
        }
    }

    Ok(())
}

/*
 * ─────────────────────────────────────
 * SEARCH
 * ─────────────────────────────────────
 */

async fn run_search() -> Result<()> {
    let config = cli::setup();

    println!();

    println!(
        "──────────────────────────────────────────"
    );

    println!("Wally configuration:");

    println!(
        "──────────────────────────────────────────"
    );

    println!(
        "📁 Directory:   {}",
        config.directory
    );

    println!(
        "🌐 Source:      {}",
        config.source
    );

    println!(
        "🔞 Rating:      {}",
        config.rating
    );

    println!(
        "🖥 Resolution:  {}",
        config.resolution
    );

    if config.tags.is_empty() {
        println!("🏷 Tags:        Any");
    } else {
        println!(
            "🏷 Tags:        {}",
            config.tags
        );
    }

    println!(
        "📦 Amount:      {}",
        config.amount
    );

    println!(
        "──────────────────────────────────────────"
    );

    let wallpapers =
        match config.source {

            cli::Source::Wallhaven => {
                sources::wallhaven::search(
                    &config
                )
                .await?
                .data
            }

            cli::Source::Konachan => {
                sources::konachan::search(
                    &config
                )
                .await?
            }

            cli::Source::Both => {
                search_both(
                    &config
                )
                .await?
            }
        };


    if wallpapers.is_empty() {
        anyhow::bail!(
            "No wallpapers found."
        );
    }


    let previews =
        downloader::load_previews(
            &wallpapers
        )
        .await?;


    ui::run(
        wallpapers,
        previews,
        config.directory.clone(),
    )
    .await?;


    Ok(())
}

/*
 * ─────────────────────────────────────
 * RANDOM
 * ─────────────────────────────────────
 */

async fn run_random() -> Result<()> {
    let mut random_config = cli::setup();

    // Random получает одну картинку за один API-запрос.
    random_config.amount = 1;

    ui::run_random(random_config).await?;

    Ok(())
}

/*
 * ─────────────────────────────────────
 * BOTH SOURCES
 * ─────────────────────────────────────
 */

pub async fn search_both(
    config: &cli::WallyConfig,
) -> Result<Vec<sources::Wallpaper>> {

    println!();

    println!(
        "🌐 Searching both sources..."
    );


    let requested =
        config.amount;


    /*
     * Делим количество примерно
     * пополам между источниками.
     */

    let wallhaven_amount =
        if requested <= 1 {
            1
        } else {
            requested / 2
        };


    let konachan_amount =
        requested
            .saturating_sub(
                wallhaven_amount
            );


    let mut results =
        Vec::new();


    /*
     * ─────────────────────────────
     * WALLHAVEN
     * ─────────────────────────────
     */

    if wallhaven_amount > 0 {

        let wallhaven_config =
            clone_config_with_amount(
                config,
                wallhaven_amount,
            );


        match sources::wallhaven::search(
            &wallhaven_config
        )
        .await
        {
            Ok(response) => {
                results.extend(
                    response.data
                );
            }

            Err(error) => {
                println!();

                println!(
                    "⚠ Wallhaven search failed:"
                );

                println!(
                    "  {}",
                    error
                );
            }
        }
    }


    /*
     * ─────────────────────────────
     * KONACHAN
     * ─────────────────────────────
     */

    if konachan_amount > 0 {

        let konachan_config =
            clone_config_with_amount(
                config,
                konachan_amount,
            );


        match sources::konachan::search(
            &konachan_config
        )
        .await
        {
            Ok(response) => {
                results.extend(
                    response
                );
            }

            Err(error) => {
                println!();

                println!(
                    "⚠ Konachan search failed:"
                );

                println!(
                    "  {}",
                    error
                );
            }
        }
    }


    /*
     * Ограничиваем итоговое количество.
     */

    results.truncate(
        requested as usize
    );


    if results.is_empty() {

        anyhow::bail!(
            "No new wallpapers found from either source."
        );
    }


    println!();

    println!(
        "✓ Combined results: {} wallpapers",
        results.len()
    );


    Ok(results)
}

/*
 * ─────────────────────────────────────
 * CONFIG CLONE
 * ─────────────────────────────────────
 */

fn clone_config_with_amount(
    config: &cli::WallyConfig,
    amount: u32,
) -> cli::WallyConfig {

    cli::WallyConfig {
        directory:
            config.directory.clone(),

        source:
            config.source,

        rating:
            config.rating,

        resolution:
            config.resolution.clone(),

        tags:
            config.tags.clone(),

        amount,
    }
}

/*
 * ─────────────────────────────────────
 * HISTORY
 * ─────────────────────────────────────
 */

fn run_history(
    action: Option<HistoryAction>,
) -> Result<()> {

    match action {

        None => {
            show_history();
        }

        Some(HistoryAction::Clear) => {
            clear_history()?;
        }

        Some(HistoryAction::Remove { id }) => {
            remove_history(&id)?;
        }
    }


    Ok(())
}

/*
 * ─────────────────────────────────────
 * SHOW HISTORY
 * ─────────────────────────────────────
 */

fn show_history() {
    let entries =
        history::load();


    println!();

    println!(
        "╭──────────────────────────────────────────╮"
    );

    println!(
        "│              WALLY HISTORY               │"
    );

    println!(
        "╰──────────────────────────────────────────╯"
    );

    println!();


    if entries.is_empty() {

        println!(
            "History is empty."
        );

        println!();

        return;
    }


    println!(
        "Downloaded: {} wallpapers",
        entries.len()
    );

    println!();


    for (index, entry)
        in entries.iter().enumerate()
    {

        println!(
            "{:>3}. {}:{}",
            index + 1,
            entry.source,
            entry.id
        );


        println!(
            "     {}x{} • {}",
            entry.width,
            entry.height,
            entry.file_type
        );


        println!(
            "     {}",
            entry.path
        );


        println!();
    }
}

/*
 * ─────────────────────────────────────
 * CLEAR HISTORY
 * ─────────────────────────────────────
 */

fn clear_history() -> Result<()> {
    let entries =
        history::load();


    if entries.is_empty() {

        println!(
            "History is already empty."
        );

        return Ok(());
    }


    history::clear()?;


    println!(
        "✓ History cleared."
    );


    println!(
        "Removed {} entries.",
        entries.len()
    );


    Ok(())
}

/*
 * ─────────────────────────────────────
 * REMOVE HISTORY ENTRY
 * ─────────────────────────────────────
 */

fn remove_history(
    id: &str,
) -> Result<()> {

    let entries =
        history::load();


    let matches =
        entries
            .iter()
            .filter(
                |entry| entry.id == id
            )
            .collect::<Vec<_>>();


    if matches.is_empty() {

        println!(
            "⚠ Wallpaper '{}' is not in history.",
            id
        );

        return Ok(());
    }


    /*
     * Если ID существует только один раз,
     * удаляем сразу.
     */

    if matches.len() == 1 {

        history::remove(id)?;


        println!(
            "✓ Removed '{}' from history.",
            id
        );


        return Ok(());
    }


    /*
     * Теоретически одинаковый ID может
     * существовать у разных источников.
     */

    println!(
        "⚠ Multiple sources contain ID '{}'.",
        id
    );


    for entry in matches {

        println!(
            "  {}:{}",
            entry.source,
            entry.id
        );
    }


    println!();

    println!(
        "Remove command currently removes all entries with this ID."
    );


    history::remove(id)?;


    println!(
        "✓ Removed all '{}' entries.",
        id
    );


    Ok(())
}