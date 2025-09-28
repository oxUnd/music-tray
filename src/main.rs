use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, MouseButton, MouseEvent, MouseEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Gauge, Paragraph},
    Frame, Terminal,
};
use std::{
    io,
    time::{Duration, Instant},
    collections::HashMap,
    fs::OpenOptions,
    path::Path,
};
use viuer;
use log::{info, error};

mod music;
use music::MusicPlayer;

// Initialize logging to file
fn init_logging() -> Result<()> {
    let log_file = "music-tray.log";
    
    // Create log file if it doesn't exist
    if !Path::new(log_file).exists() {
        std::fs::File::create(log_file)?;
    }
    
    // Configure env_logger to write to file
    env_logger::Builder::from_default_env()
        .target(env_logger::Target::Pipe(Box::new(
            OpenOptions::new()
                .create(true)
                .append(true)
                .open(log_file)?
        )))
        .init();
    
    info!("Music Tray application started");
    Ok(())
}


#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    init_logging()?;
    
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    info!("Terminal setup completed");

    // Create app and run
    let app = App::new();
    let res = run_app(&mut terminal, app).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        error!("Application error: {err:?}");
    }

    info!("Music Tray application ended");
    Ok(())
}

struct App {
    music_player: MusicPlayer,
    should_quit: bool,
    button_positions: HashMap<String, (u16, u16, u16, u16)>, // button_name -> (x, y, width, height)
    image_display_enabled: bool, // Track if terminal supports image display
}

impl App {
    fn new() -> Self {
        Self {
            music_player: MusicPlayer::new(),
            should_quit: false,
            button_positions: HashMap::new(),
            image_display_enabled: viuer::is_iterm_supported(),
        }
    }

    fn on_tick(&mut self) {
        // Update music player state
        self.music_player.update();
    }

    fn on_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char('q') => {
                self.should_quit = true;
            }
            KeyCode::Char(' ') => {
                self.music_player.toggle_play_pause();
            }
            KeyCode::Char('n') => {
                self.music_player.next();
            }
            KeyCode::Char('p') => {
                self.music_player.previous();
            }
            _ => {}
        }
    }

    fn on_mouse(&mut self, mouse: MouseEvent) {
        if let MouseEventKind::Down(MouseButton::Left) = mouse.kind {
            self.handle_button_click(mouse.column, mouse.row);
        }
    }

    fn handle_button_click(&mut self, x: u16, y: u16) {
        // Check if click is within any button area
        for (button_name, (btn_x, btn_y, btn_width, btn_height)) in &self.button_positions {
            if x >= *btn_x && x < *btn_x + *btn_width && 
               y >= *btn_y && y < *btn_y + *btn_height {
                match button_name.as_str() {
                    "previous" => {
                        self.music_player.previous();
                        info!("Previous button clicked");
                    }
                    "play_pause" => {
                        self.music_player.toggle_play_pause();
                        info!("Play/Pause button clicked");
                    }
                    "next" => {
                        self.music_player.next();
                        info!("Next button clicked");
                    }
                    "quit" => {
                        self.should_quit = true;
                        info!("Quit button clicked");
                    }
                    _ => {}
                }
                break;
            }
        }
    }
}

async fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    let mut last_tick = Instant::now();
    let tick_rate = Duration::from_millis(250);

    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if crossterm::event::poll(timeout)? {
            match event::read()? {
                Event::Key(key) => {
                    if key.kind == KeyEventKind::Press {
                        app.on_key(key.code);
                    }
                }
                Event::Mouse(mouse) => {
                    app.on_mouse(mouse);
                }
                _ => {}
            }
        }

        if last_tick.elapsed() >= tick_rate {
            app.on_tick();
            last_tick = Instant::now();
        }

        if app.should_quit {
            break;
        }
    }

    Ok(())
}

fn ui(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Min(10),   // Main content
            Constraint::Length(5), // Controls
        ])
        .split(f.size());

    // Title
    let title = Paragraph::new("🎵 Music Tray")
        .style(Style::default().fg(Color::Yellow))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);

    // Main content area
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(chunks[1]);

    // Cover art area (left side) - Show simple placeholder for now
    let cover_content = if app.music_player.get_current_track().cover_url.is_some() {
        if app.image_display_enabled {
            "🖼️\n\nImage Display\nSupported\n\n🖼️".to_string()
        } else {
            "🖼️\n\nImage Display\nNot Supported\n\n🖼️".to_string()
        }
    } else {
        "🎵\n\nNo Cover\nAvailable\n\n🎵".to_string()
    };
    
    let cover_title = if app.image_display_enabled {
        "🎨 Cover Art (Image Display)"
    } else {
        "🎨 Cover Art (Text Mode)"
    };
    
    let cover_block = Paragraph::new(cover_content)
        .style(Style::default().fg(Color::Blue))
        .alignment(Alignment::Center)
        .block(Block::default()
            .borders(Borders::ALL)
            .title(cover_title)
            .title_style(Style::default().fg(Color::Yellow)));
    f.render_widget(cover_block, main_chunks[0]);

    // Track info area (right side)
    let track_info = app.music_player.get_current_track();
    let track_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // Connection status
            Constraint::Length(3), // Track name
            Constraint::Length(3), // Artist
            Constraint::Length(3), // Album
            Constraint::Length(3), // Progress
            Constraint::Min(0),    // Spacer
        ])
        .split(main_chunks[1]);

    // Connection status and player info
    let connection_status = if app.music_player.is_connected() {
        if let Some(player) = app.music_player.get_current_player() {
            let player_name = player.identity();
            format!("🔗 Connected to: {}", player_name)
        } else {
            "🔗 Connected (No active player)".to_string()
        }
    } else {
        "❌ Not connected to D-Bus".to_string()
    };
    
    let status_block = Paragraph::new(connection_status)
        .style(Style::default().fg(Color::Green))
        .block(Block::default().borders(Borders::ALL).title("🔌 Status"));
    f.render_widget(status_block, track_chunks[0]);

    // Track name with play status
    let play_status = if track_info.is_playing { "▶️" } else { "⏸️" };
    let track_name = Paragraph::new(format!("{} {}", play_status, track_info.title.as_deref().unwrap_or("Unknown")))
        .style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::ALL).title("🎵 Track"));
    f.render_widget(track_name, track_chunks[1]);

    // Artist
    let artist = Paragraph::new(track_info.artist.as_deref().unwrap_or("Unknown Artist"))
        .style(Style::default().fg(Color::Cyan))
        .block(Block::default().borders(Borders::ALL).title("🎤 Artist"));
    f.render_widget(artist, track_chunks[2]);

    // Album
    let album = Paragraph::new(track_info.album.as_deref().unwrap_or("Unknown Album"))
        .style(Style::default().fg(Color::Magenta))
        .block(Block::default().borders(Borders::ALL).title("💿 Album"));
    f.render_widget(album, track_chunks[3]);

    // Progress bar with time display
    let progress = if track_info.duration > 0 {
        (track_info.position as f64 / track_info.duration as f64 * 100.0) as u16
    } else {
        0
    };
    
    let position_str = format!("{:02}:{:02}", track_info.position / 60, track_info.position % 60);
    let duration_str = format!("{:02}:{:02}", track_info.duration / 60, track_info.duration % 60);
    let progress_text = format!("{} / {}", position_str, duration_str);
    
    let progress_gauge = Gauge::default()
        .block(Block::default()
            .borders(Borders::ALL)
            .title(format!("⏱️  Progress ({})", progress_text)))
        .gauge_style(Style::default().fg(Color::Green))
        .percent(progress);
    f.render_widget(progress_gauge, track_chunks[4]);

    // Clickable Controls
    let control_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25), // Previous button
            Constraint::Percentage(25), // Play/Pause button
            Constraint::Percentage(25), // Next button
            Constraint::Percentage(25), // Quit button
        ])
        .split(chunks[2]);

    // Store button positions for mouse click detection
    app.button_positions.insert("previous".to_string(), 
        (control_chunks[0].x, control_chunks[0].y, control_chunks[0].width, control_chunks[0].height));
    app.button_positions.insert("play_pause".to_string(), 
        (control_chunks[1].x, control_chunks[1].y, control_chunks[1].width, control_chunks[1].height));
    app.button_positions.insert("next".to_string(), 
        (control_chunks[2].x, control_chunks[2].y, control_chunks[2].width, control_chunks[2].height));
    app.button_positions.insert("quit".to_string(), 
        (control_chunks[3].x, control_chunks[3].y, control_chunks[3].width, control_chunks[3].height));

    // Previous button
    let prev_button = Paragraph::new("⏮️ Previous")
        .style(Style::default().fg(Color::White).bg(Color::Blue))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).title("P"));
    f.render_widget(prev_button, control_chunks[0]);

    // Play/Pause button
    let play_pause_text = if track_info.is_playing { "⏸️ Pause" } else { "▶️ Play" };
    let play_pause_button = Paragraph::new(play_pause_text)
        .style(Style::default().fg(Color::White).bg(Color::Green))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).title("SPACE"));
    f.render_widget(play_pause_button, control_chunks[1]);

    // Next button
    let next_button = Paragraph::new("Next ⏭️")
        .style(Style::default().fg(Color::White).bg(Color::Blue))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).title("N"));
    f.render_widget(next_button, control_chunks[2]);

    // Quit button
    let quit_button = Paragraph::new("❌ Quit")
        .style(Style::default().fg(Color::White).bg(Color::Red))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).title("Q"));
    f.render_widget(quit_button, control_chunks[3]);
}


