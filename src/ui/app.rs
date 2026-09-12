use std::io::{self, Stdout};

use anyhow::{Context, Result};

use crossterm::{
    cursor,
    event::{self, Event, KeyCode},
    execute,
    terminal::{
        disable_raw_mode,
        enable_raw_mode,
        EnterAlternateScreen,
        LeaveAlternateScreen,
    },
};

use ratatui::{
    backend::CrosstermBackend,
    layout::{
        Constraint,
        Direction,
        Layout,
    },
    style::Style,
    widgets::{
        Block,
        Borders,
        Paragraph,
    },
    Terminal,
};

use ratatui_image::{
    picker::Picker,
    protocol::StatefulProtocol,
    Resize,
    StatefulImage,
};

use image::DynamicImage;

use crate::{
    cli::WallyConfig,
    downloader,
    sources::Wallpaper,
    wallpaper,
};


pub struct App {
    wallpapers: Vec<Wallpaper>,
    images: Vec<StatefulProtocol>,

    current: usize,

    selected: Vec<bool>,

    directory: String,
}


impl App {
    pub fn new(
        wallpapers: Vec<Wallpaper>,
        images: Vec<StatefulProtocol>,
        directory: String,
    ) -> Self {

        let selected =
            vec![false; wallpapers.len()];

        Self {
            wallpapers,
            images,
            current: 0,
            selected,
            directory,
        }
    }


    /*
     * ─────────────────────────────
     * NAVIGATION
     * ─────────────────────────────
     */

    fn next(&mut self) {

        if self.wallpapers.is_empty() {
            return;
        }

        self.current =
            (self.current + 1)
                % self.wallpapers.len();
    }


    fn previous(&mut self) {

        if self.wallpapers.is_empty() {
            return;
        }

        if self.current == 0 {

            self.current =
                self.wallpapers.len() - 1;

        } else {

            self.current -= 1;

        }
    }


    /*
     * ─────────────────────────────
     * SELECTION
     * ─────────────────────────────
     */

    fn toggle_selected(&mut self) {

        if let Some(value) =
            self.selected
                .get_mut(self.current)
        {

            *value = !*value;

        }
    }


    fn selected_count(&self) -> usize {

        self.selected
            .iter()
            .filter(|&&value| value)
            .count()

    }


    /*
     * ─────────────────────────────
     * CURRENT WALLPAPER
     * ─────────────────────────────
     */

    fn current_wallpaper(
        &self,
    ) -> Option<&Wallpaper> {

        self.wallpapers
            .get(self.current)

    }


    /*
     * ─────────────────────────────
     * DOWNLOAD TARGETS
     * ─────────────────────────────
     */

    fn download_targets(
        &self,
    ) -> Vec<&Wallpaper> {

        /*
         * Если ничего не выбрано,
         * скачиваем текущие обои.
         */

        if self.selected_count() == 0 {

            return self
                .current_wallpaper()
                .into_iter()
                .collect();

        }


        /*
         * Если есть выбранные —
         * скачиваем только их.
         */

        self.wallpapers
            .iter()
            .enumerate()
            .filter_map(
                |(index, wallpaper)| {

                    if self.selected[index] {

                        Some(wallpaper)

                    } else {

                        None

                    }

                },
            )
            .collect()

    }
}


/*
 * ─────────────────────────────────────
 * START
 * ─────────────────────────────────────
 */

pub async fn run_random(
    config: WallyConfig,
) -> Result<()> {
    let picker =
        Picker::from_query_stdio()
            .context(
                "Failed to detect terminal graphics protocol",
            )?;

    enable_raw_mode()?;

    let mut stdout = io::stdout();

    execute!(
        stdout,
        EnterAlternateScreen,
        cursor::Hide
    )?;

    let backend =
        CrosstermBackend::new(stdout);

    let mut terminal =
        Terminal::new(backend)?;

    terminal.clear()?;

    let result =
        run_random_app(
            &mut terminal,
            config,
            picker,
        )
        .await;

    disable_raw_mode()?;

    execute!(
        terminal.backend_mut(),
        cursor::Show,
        LeaveAlternateScreen
    )?;

    terminal.show_cursor()?;

    result
}


/*
 * ─────────────────────────────────────
 * RANDOM MAIN LOOP
 * ─────────────────────────────────────
 */

async fn run_random_app(
    terminal:
        &mut Terminal<
            CrosstermBackend<Stdout>
        >,
    config: WallyConfig,
    picker: Picker,
) -> Result<()> {
    let mut wallpapers: Vec<Wallpaper> = Vec::new();
    let mut images: Vec<StatefulProtocol> = Vec::new();

    loop {
        /*
         * Получаем одну новую картинку
         * при каждом запросе.
         */
        let wallpaper =
            fetch_random_wallpaper(&config).await?;

        let loaded =
            downloader::load_previews(
                std::slice::from_ref(&wallpaper),
            )
            .await?;

        if loaded.is_empty() {
            anyhow::bail!(
                "Failed to load wallpaper preview."
            );
        }

        let protocol =
            picker.new_resize_protocol(
                loaded.into_iter().next().unwrap(),
            );

        wallpapers.clear();
        images.clear();

        wallpapers.push(wallpaper);
        images.push(protocol);

        /*
         * Показываем текущую случайную картинку
         * и ждём действия пользователя.
         */
        loop {
            terminal.draw(|frame| {
                let area = frame.area();

                let chunks =
                    Layout::default()
                        .direction(
                            Direction::Vertical,
                        )
                        .constraints([
                            Constraint::Length(3),
                            Constraint::Min(1),
                            Constraint::Length(6),
                        ])
                        .split(area);

                let wallpaper =
                    wallpapers.first();

                let header =
                    Paragraph::new(
                        " WALLY  •  RANDOM ",
                    )
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title(
                                " Random Wallpaper ",
                            ),
                    );

                frame.render_widget(
                    header,
                    chunks[0],
                );

                let image_block =
                    Block::default()
                        .borders(Borders::ALL)
                        .title(" Preview ");

                let image_area =
                    image_block.inner(chunks[1]);

                frame.render_widget(
                    image_block,
                    chunks[1],
                );

                if let Some(protocol) =
                    images.first_mut()
                {
                    let image =
                        StatefulImage::new()
                            .resize(
                                Resize::Fit(None),
                            );

                    frame.render_stateful_widget(
                        image,
                        image_area,
                        protocol,
                    );
                }

                let footer_text =
                    if let Some(wallpaper) =
                        wallpaper
                    {
                        format!(
                            " ID: {}    {} × {}    {}\n\
                             \n\
                             N / → Next random    \
                             ENTER Download    \
                             D Download    \
                             S Set wallpaper    \
                             Q Quit",
                            wallpaper.id,
                            wallpaper.dimension_x,
                            wallpaper.dimension_y,
                            wallpaper.file_type,
                        )
                    } else {
                        "No wallpaper found."
                            .to_string()
                    };

                let footer =
                    Paragraph::new(
                        footer_text,
                    )
                    .block(
                        Block::default()
                            .borders(
                                Borders::ALL,
                            )
                            .title(
                                " Controls ",
                            ),
                    );

                frame.render_widget(
                    footer,
                    chunks[2],
                );
            })?;

            if event::poll(
                std::time::Duration::from_millis(
                    100,
                ),
            )? {
                if let Event::Key(key) =
                    event::read()?
                {
                    match key.code {
                        /*
                         * NEXT RANDOM
                         *
                         * Выходим из внутреннего
                         * цикла и делаем НОВЫЙ
                         * запрос к API.
                         */
                        KeyCode::Char('n')
                        | KeyCode::Char('N')
                        | KeyCode::Right => {
                            break;
                        }

                        /*
                         * DOWNLOAD
                         */
                        KeyCode::Enter
                        | KeyCode::Char('d')
                        | KeyCode::Char('D') => {
                            if let Some(wallpaper) =
                                wallpapers.first()
                            {
                                download_random_current(
                                    wallpaper,
                                    &config.directory,
                                )
                                .await?;

                                terminal.clear()?;
                            }
                        }

                        /*
                         * DOWNLOAD + SET
                         */
                        KeyCode::Char('s')
                        | KeyCode::Char('S') => {
                            if let Some(wallpaper) =
                                wallpapers.first()
                            {
                                set_random_current(
                                    wallpaper,
                                    &config.directory,
                                )
                                .await?;

                                terminal.clear()?;
                            }
                        }

                        /*
                         * QUIT
                         */
                        KeyCode::Char('q')
                        | KeyCode::Char('Q')
                        | KeyCode::Esc => {
                            return Ok(());
                        }

                        _ => {}
                    }
                }
            }
        }
    }
}


/*
 * ─────────────────────────────────────
 * RANDOM API REQUEST
 * ─────────────────────────────────────
 */

async fn fetch_random_wallpaper(
    config: &WallyConfig,
) -> Result<Wallpaper> {
    let mut random_config =
        config.clone();

    random_config.amount = 1;

    let wallpapers = match random_config.source {
        crate::cli::Source::Wallhaven => {
            crate::sources::wallhaven::search(
                &random_config,
            )
            .await?
            .data
        }

        crate::cli::Source::Konachan => {
            crate::sources::konachan::search(
                &random_config,
            )
            .await?
        }

        crate::cli::Source::Both => {
            crate::search_both(
                &random_config,
            )
            .await?
        }
    };

    wallpapers
        .into_iter()
        .next()
        .ok_or_else(|| {
            anyhow::anyhow!(
                "No new random wallpapers found."
            )
        })
}


/*
 * ─────────────────────────────────────
 * RANDOM DOWNLOAD
 * ─────────────────────────────────────
 */

async fn download_random_current(
    wallpaper: &Wallpaper,
    directory: &str,
) -> Result<()> {
    disable_raw_mode()?;

    let mut stdout =
        io::stdout();

    execute!(
        stdout,
        cursor::Show,
        LeaveAlternateScreen
    )?;

    println!();
    println!(
        "⬇ Downloading wallpaper {}...",
        wallpaper.id
    );

    downloader::download_wallpaper(
        wallpaper,
        directory,
    )
    .await?;

    println!();
    println!(
        "✓ Download complete!"
    );
    println!();
    println!(
        "Press ENTER to return to Wally..."
    );

    let mut input =
        String::new();

    io::stdin()
        .read_line(&mut input)
        .ok();

    enable_raw_mode()?;

    execute!(
        stdout,
        EnterAlternateScreen,
        cursor::Hide
    )?;

    Ok(())
}


/*
 * ─────────────────────────────────────
 * RANDOM SET WALLPAPER
 * ─────────────────────────────────────
 */

async fn set_random_current(
    wallpaper: &Wallpaper,
    directory: &str,
) -> Result<()> {
    disable_raw_mode()?;

    let mut stdout =
        io::stdout();

    execute!(
        stdout,
        cursor::Show,
        LeaveAlternateScreen
    )?;

    println!();
    println!(
        "⬇ Downloading wallpaper {}...",
        wallpaper.id
    );

    let path =
        downloader::download_wallpaper(
            wallpaper,
            directory,
        )
        .await?;

    println!();
    println!(
        "🖥 Setting wallpaper..."
    );

    wallpaper::set_wallpaper(
        &path,
    )
    .await?;

    println!();
    println!(
        "✓ Wallpaper set successfully!"
    );
    println!();
    println!(
        "Press ENTER to return to Wally..."
    );

    let mut input =
        String::new();

    io::stdin()
        .read_line(&mut input)
        .ok();

    enable_raw_mode()?;

    execute!(
        stdout,
        EnterAlternateScreen,
        cursor::Hide
    )?;

    Ok(())
}


pub async fn run(
    wallpapers: Vec<Wallpaper>,
    images: Vec<DynamicImage>,
    directory: String,
) -> Result<()> {

    /*
     * Определяем возможности терминала.
     */

    let picker =
        Picker::from_query_stdio()
            .context(
                "Failed to detect terminal graphics protocol",
            )?;


    /*
     * Создаём image protocols.
     */

    let protocols =
        images
            .into_iter()
            .map(
                |image| {
                    picker.new_resize_protocol(image)
                },
            )
            .collect::<Vec<_>>();


    enable_raw_mode()?;


    let mut stdout =
        io::stdout();


    execute!(
        stdout,
        EnterAlternateScreen,
        cursor::Hide
    )?;


    let backend =
        CrosstermBackend::new(stdout);


    let mut terminal =
        Terminal::new(backend)?;


    terminal.clear()?;


    let result =
        run_app(
            &mut terminal,
            wallpapers,
            protocols,
            directory,
        )
        .await;


    /*
     * Восстанавливаем терминал.
     */

    disable_raw_mode()?;


    execute!(
        terminal.backend_mut(),
        cursor::Show,
        LeaveAlternateScreen
    )?;


    terminal.show_cursor()?;


    result
}


/*
 * ─────────────────────────────────────
 * MAIN LOOP
 * ─────────────────────────────────────
 */

async fn run_app(
    terminal:
        &mut Terminal<
            CrosstermBackend<Stdout>
        >,

    wallpapers:
        Vec<Wallpaper>,

    images:
        Vec<StatefulProtocol>,

    directory:
        String,
) -> Result<()> {

    let mut app =
        App::new(
            wallpapers,
            images,
            directory,
        );


    loop {

        /*
         * ─────────────────────────
         * DRAW
         * ─────────────────────────
         */

        terminal.draw(
            |frame| {

                let area =
                    frame.area();


                /*
                 * Header / Image / Footer
                 */

                let chunks =
                    Layout::default()
                        .direction(
                            Direction::Vertical
                        )
                        .constraints([
                            Constraint::Length(3),
                            Constraint::Min(1),
                            Constraint::Length(6),
                        ])
                        .split(area);


                /*
                 * ─────────────────
                 * HEADER
                 * ─────────────────
                 */

                let position =
                    if app.wallpapers.is_empty() {

                        "0 / 0".to_string()

                    } else {

                        format!(
                            "{} / {}",
                            app.current + 1,
                            app.wallpapers.len()
                        )

                    };


                let header =
                    Paragraph::new(
                        format!(
                            " WALLY  •  Wallhaven                         {} ",
                            position
                        ),
                    )
                    .block(
                        Block::default()
                            .borders(
                                Borders::ALL
                            )
                            .title(
                                " Wallpaper Browser "
                            ),
                    );


                frame.render_widget(
                    header,
                    chunks[0],
                );


                /*
                 * ─────────────────
                 * IMAGE
                 * ─────────────────
                 */

                let image_block =
                    Block::default()
                        .borders(
                            Borders::ALL
                        )
                        .title(
                            " Preview "
                        );


                let image_area =
                    image_block.inner(
                        chunks[1]
                    );


                frame.render_widget(
                    image_block,
                    chunks[1],
                );


                if let Some(protocol) =
                    app.images
                        .get_mut(app.current)
                {

                    let image =
                        StatefulImage::new()
                            .resize(
                                Resize::Fit(None)
                            );


                    frame.render_stateful_widget(
                        image,
                        image_area,
                        protocol,
                    );

                }


                /*
                 * ─────────────────
                 * FOOTER
                 * ─────────────────
                 */

                let wallpaper =
                    app.current_wallpaper();


                let footer_text =
                    if let Some(wallpaper) =
                        wallpaper
                    {

                        let status =
                            if app.selected[
                                app.current
                            ] {

                                "✓ SELECTED"

                            } else {

                                "Not selected"

                            };


                        format!(
                            " ID: {}    {} × {}    {}\n\
                             Status: {}    Selected: {}\n\
                             \n\
                             ← → Navigate    \
                             SPACE Select    \
                             ENTER Download    \
                             D Download current    \
                             S Set wallpaper    \
                             Q Quit",

                            wallpaper.id,

                            wallpaper.dimension_x,

                            wallpaper.dimension_y,

                            wallpaper.file_type,

                            status,

                            app.selected_count(),
                        )

                    } else {

                        "No wallpapers found."
                            .to_string()

                    };


                let footer =
                    Paragraph::new(
                        footer_text
                    )
                    .style(
                        Style::default()
                    )
                    .block(
                        Block::default()
                            .borders(
                                Borders::ALL
                            )
                            .title(
                                " Controls "
                            ),
                    );


                frame.render_widget(
                    footer,
                    chunks[2],
                );

            },
        )?;


        /*
         * ─────────────────────────
         * INPUT
         * ─────────────────────────
         */

        if event::poll(
            std::time::Duration::from_millis(
                100
            ),
        )? {

            if let Event::Key(key) =
                event::read()?
            {

                match key.code {

                    /*
                     * PREVIOUS
                     */

                    KeyCode::Left |
                    KeyCode::Char('a') => {

                        app.previous();

                    }


                    /*
                     * NEXT
                     */

                    KeyCode::Right => {

                        app.next();

                    }


                    /*
                     * SELECT
                     */

                    KeyCode::Char(' ') => {

                        app.toggle_selected();

                    }


                    /*
                     * ENTER
                     *
                     * Download selected/current.
                     */

                    KeyCode::Enter => {

                        download_selected(
                            &app
                        )
                        .await?;

                        terminal.clear()?;

                    }


                    /*
                     * D
                     *
                     * Download current.
                     */

                    KeyCode::Char('D') |
                    KeyCode::Char('d') => {

                        download_current(
                            &app
                        )
                        .await?;

                        terminal.clear()?;

                    }


                    /*
                     * S
                     *
                     * Download current
                     * and set as wallpaper.
                     */

                    KeyCode::Char('S') |
                    KeyCode::Char('s') => {

                        set_current_wallpaper(
                            &app
                        )
                        .await?;

                        terminal.clear()?;

                    }


                    /*
                     * QUIT
                     */

                    KeyCode::Char('q') |
                    KeyCode::Char('Q') |
                    KeyCode::Esc => {

                        break;

                    }


                    _ => {}

                }

            }

        }

    }


    Ok(())
}


/*
 * ─────────────────────────────────────
 * DOWNLOAD SELECTED
 * ─────────────────────────────────────
 */

async fn download_selected(
    app: &App,
) -> Result<()> {

    disable_raw_mode()?;


    let mut stdout =
        io::stdout();


    execute!(
        stdout,
        cursor::Show,
        LeaveAlternateScreen
    )?;


    println!();

    println!(
        "╭──────────────────────────────────────────╮"
    );

    println!(
        "│              WALLY DOWNLOAD              │"
    );

    println!(
        "╰──────────────────────────────────────────╯"
    );

    println!();


    println!(
        "📁 Directory: {}",
        app.directory
    );

    println!();


    let targets =
        app.download_targets();


    println!(
        "⬇ Downloading {} wallpaper(s)...",
        targets.len()
    );

    println!();


    for wallpaper in targets {

        downloader::download_wallpaper(
            wallpaper,
            &app.directory,
        )
        .await?;

    }


    println!();

    println!(
        "✓ Download complete!"
    );

    println!();

    println!(
        "Press ENTER to return to Wally..."
    );


    let mut input =
        String::new();


    io::stdin()
        .read_line(&mut input)
        .ok();


    /*
     * Возвращаем TUI.
     */

    enable_raw_mode()?;


    execute!(
        stdout,
        EnterAlternateScreen,
        cursor::Hide
    )?;


    Ok(())
}


/*
 * ─────────────────────────────────────
 * DOWNLOAD CURRENT
 * ─────────────────────────────────────
 */

async fn download_current(
    app: &App,
) -> Result<()> {

    let Some(wallpaper) =
        app.current_wallpaper()
    else {

        return Ok(());

    };


    disable_raw_mode()?;


    let mut stdout =
        io::stdout();


    execute!(
        stdout,
        cursor::Show,
        LeaveAlternateScreen
    )?;


    println!();

    println!(
        "╭──────────────────────────────────────────╮"
    );

    println!(
        "│          WALLY QUICK DOWNLOAD            │"
    );

    println!(
        "╰──────────────────────────────────────────╯"
    );

    println!();


    println!(
        "📁 Directory: {}",
        app.directory
    );


    println!(
        "🖼 Wallpaper: {}",
        wallpaper.id
    );


    println!();


    downloader::download_wallpaper(
        wallpaper,
        &app.directory,
    )
    .await?;


    println!();

    println!(
        "✓ Download complete!"
    );

    println!();

    println!(
        "Press ENTER to return to Wally..."
    );


    let mut input =
        String::new();


    io::stdin()
        .read_line(&mut input)
        .ok();


    /*
     * Возвращаем TUI.
     */

    enable_raw_mode()?;


    execute!(
        stdout,
        EnterAlternateScreen,
        cursor::Hide
    )?;


    Ok(())
}


/*
 * ─────────────────────────────────────
 * SET CURRENT WALLPAPER
 * ─────────────────────────────────────
 */

async fn set_current_wallpaper(
    app: &App,
) -> Result<()> {

    let Some(wallpaper) =
        app.current_wallpaper()
    else {

        return Ok(());

    };


    disable_raw_mode()?;


    let mut stdout =
        io::stdout();


    execute!(
        stdout,
        cursor::Show,
        LeaveAlternateScreen
    )?;


    println!();

    println!(
        "╭──────────────────────────────────────────╮"
    );

    println!(
        "│             WALLY WALLPAPER              │"
    );

    println!(
        "╰──────────────────────────────────────────╯"
    );

    println!();


    println!(
        "📁 Directory: {}",
        app.directory
    );

    println!(
        "🖼 Wallpaper: {}",
        wallpaper.id
    );

    println!();


    /*
     * Сначала скачиваем оригинал.
     */

    println!("⬇ Downloading wallpaper...");

    let path =
        downloader::download_wallpaper(
            wallpaper,
            &app.directory,
        )
        .await?;


    println!();


    /*
     * Передаём локальный файл
     * в wallpaper backend.
     */

    println!("🖥 Setting wallpaper...");

    wallpaper::set_wallpaper(
        &path
    )
    .await?;


    println!();

    println!(
        "✓ Wallpaper set successfully!"
    );

    println!();

    println!(
        "Press ENTER to return to Wally..."
    );


    let mut input =
        String::new();


    io::stdin()
        .read_line(&mut input)
        .ok();


    /*
     * Возвращаем TUI.
     */

    enable_raw_mode()?;


    execute!(
        stdout,
        EnterAlternateScreen,
        cursor::Hide
    )?;


    Ok(())
}