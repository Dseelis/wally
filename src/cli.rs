use dialoguer::{
    Confirm,
    Input,
    Select,
};

use crate::locations;

#[derive(Debug, Clone)]
pub struct WallyConfig {
    pub directory: String,
    pub source: Source,
    pub rating: Rating,
    pub resolution: Resolution,
    pub tags: String,
    pub amount: u32,
}


#[derive(Debug, Clone, Copy)]
pub enum Source {
    Wallhaven,
    Konachan,
    Both,
}


#[derive(Debug, Clone, Copy)]
pub enum Rating {
    Sfw,
    Sketchy,
    Nsfw,
}


#[derive(Debug, Clone)]
pub enum Resolution {
    Exact(u32, u32),
    Minimum(u32, u32),
    Ratio(u32, u32),
}


impl std::fmt::Display for Source {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter,
    ) -> std::fmt::Result {

        let text = match self {
            Source::Wallhaven => "Wallhaven",
            Source::Konachan => "Konachan",
            Source::Both => "Both",
        };

        write!(f, "{text}")
    }
}


impl std::fmt::Display for Rating {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter,
    ) -> std::fmt::Result {

        let text = match self {
            Rating::Sfw => "SFW",
            Rating::Sketchy => "SFW + Sketchy",
            Rating::Nsfw => "NSFW",
        };

        write!(f, "{text}")
    }
}


impl std::fmt::Display for Resolution {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter,
    ) -> std::fmt::Result {

        match self {
            Resolution::Exact(w, h) => {
                write!(f, "{w}x{h}")
            }

            Resolution::Minimum(w, h) => {
                write!(f, "Minimum {w}x{h}")
            }

            Resolution::Ratio(w, h) => {
                write!(f, "{w}:{h}")
            }
        }
    }
}



pub fn setup() -> WallyConfig {

    println!();

    println!(
        "╭──────────────────────────────────────────╮"
    );

    println!(
        "│                  WALLY                   │"
    );

    println!(
        "│           Wallpaper Downloader           │"
    );

    println!(
        "╰──────────────────────────────────────────╯"
    );

    println!();



    let directory =
        select_directory();



    let sources = [
        "Wallhaven",
        "Konachan",
        "Both",
    ];


    let source_index =
        Select::new()
            .with_prompt("Source")
            .items(&sources)
            .default(0)
            .interact()
            .expect(
                "Failed to select source"
            );


    let source =
        match source_index {
            0 => Source::Wallhaven,
            1 => Source::Konachan,
            _ => Source::Both,
        };




    let ratings = [
        "SFW",
        "SFW + Sketchy",
        "NSFW",
    ];


    let rating_index =
        Select::new()
            .with_prompt("Rating")
            .items(&ratings)
            .default(0)
            .interact()
            .expect(
                "Failed to select rating"
            );


    let rating =
        match rating_index {
            0 => Rating::Sfw,
            1 => Rating::Sketchy,
            _ => Rating::Nsfw,
        };



    let resolutions = [
        "1920x1080",
        "2560x1440",
        "3840x2160",
        "Minimum 1920x1080",
        "16:9",
        "Custom",
    ];


    let resolution_index =
        Select::new()
            .with_prompt("Resolution")
            .items(&resolutions)
            .default(0)
            .interact()
            .expect(
                "Failed to select resolution"
            );


    let resolution =
        match resolution_index {

            0 => {
                Resolution::Exact(
                    1920,
                    1080,
                )
            }

            1 => {
                Resolution::Exact(
                    2560,
                    1440,
                )
            }

            2 => {
                Resolution::Exact(
                    3840,
                    2160,
                )
            }

            3 => {
                Resolution::Minimum(
                    1920,
                    1080,
                )
            }

            4 => {
                Resolution::Ratio(
                    16,
                    9,
                )
            }

            5 => {

                let value: String =
                    Input::new()
                        .with_prompt(
                            "Enter resolution (e.g. 1920x1080)"
                        )
                        .default(
                            "1920x1080"
                                .to_string()
                        )
                        .interact_text()
                        .expect(
                            "Failed to read resolution"
                        );

                parse_resolution(
                    &value
                )
            }

            _ => unreachable!(),
        };



    let tags: String =
        Input::new()
            .with_prompt(
                "Tags (comma separated)"
            )
            .allow_empty(true)
            .interact_text()
            .expect(
                "Failed to read tags"
            );



    let amount: u32 =
        Input::new()
            .with_prompt(
                "How many wallpapers?"
            )
            .default(20)
            .validate_with(
                |value: &u32| {
                    if *value == 0 {
                        Err(
                            "Amount must be greater than 0"
                        )
                    } else {
                        Ok(())
                    }
                }
            )
            .interact_text()
            .expect(
                "Failed to read amount"
            );


    WallyConfig {
        directory,
        source,
        rating,
        resolution,
        tags,
        amount,
    }
}



fn select_directory() -> String {

    let saved =
        locations::load();


    if saved.is_empty() {

        let directory: String =
            Input::new()
                .with_prompt(
                    "Where should wallpapers be saved?"
                )
                .default(
                    "~/Pictures/Wallpapers"
                        .to_string()
                )
                .interact_text()
                .expect(
                    "Failed to read directory"
                );


        save_if_requested(
            &directory
        );


        return directory;
    }



    let mut options =
        saved.clone();

    options.push(
        "+ New location"
            .to_string()
    );


    let index =
        Select::new()
            .with_prompt(
                "Where should wallpapers be saved?"
            )
            .items(&options)
            .default(0)
            .interact()
            .expect(
                "Failed to select directory"
            );



    if index < saved.len() {

        return saved[index].clone();

    }



    let directory: String =
        Input::new()
            .with_prompt(
                "Enter wallpaper directory"
            )
            .default(
                "~/Pictures/Wallpapers"
                    .to_string()
            )
            .interact_text()
            .expect(
                "Failed to read directory"
            );


    save_if_requested(
        &directory
    );


    directory
}



fn save_if_requested(
    directory: &str,
) {

    let save =
        Confirm::new()
            .with_prompt(
                "Save this location for future use?"
            )
            .default(true)
            .interact()
            .unwrap_or(false);


    if save {

        locations::save(
            directory
        );

        println!(
            "✓ Location saved: {}",
            directory
        );
    }
}



fn parse_resolution(
    value: &str,
) -> Resolution {

    let parts:
        Vec<&str> =
        value
            .split('x')
            .collect();


    if parts.len() != 2 {

        return Resolution::Exact(
            1920,
            1080,
        );
    }


    let width =
        parts[0]
            .parse()
            .unwrap_or(1920);


    let height =
        parts[1]
            .parse()
            .unwrap_or(1080);


    Resolution::Exact(
        width,
        height,
    )
}