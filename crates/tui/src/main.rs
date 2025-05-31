use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph, Row, Table},
    Terminal,
};
use std::io;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use project::{load_project, get_project_path};
use log::{debug, error, info};
use env_logger;
use app_state::AppState;

// AppState has been moved to the app_state crate

fn main() -> Result<(), io::Error> {
    // Initialize the logger
    env_logger::init();

    std::panic::set_hook(Box::new(|info| {
        error!("Application panicked: {:?}", info);
        let _ = crossterm::terminal::disable_raw_mode();
        let _ = crossterm::execute!(
            std::io::stdout(),
            crossterm::terminal::LeaveAlternateScreen,
            crossterm::event::DisableMouseCapture
        );
    }));

    info!("Starting TUI application");

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    let project_path = get_project_path("my-song");
    
    let (_project, tracks, patterns, pattern_metas) = match load_project(project_path.clone()) {
        Ok((proj, trks, pats, metas)) => {
            info!("Project loaded successfully: {}", proj.name);
            (proj, trks, pats, metas)
        }
        Err(e) => {
            error!("Failed to load project: {}", e);
            return Err(io::Error::new(io::ErrorKind::Other, "Failed to load project"));
        }
    };

    let num_tracks = tracks.len();
    let num_steps = if !patterns.is_empty() {
        if !patterns[0].steps.is_empty() {
            patterns[0].steps[0].len()
        } else {
            16 // Default number of steps
        }
    } else {
        16 // Default number of steps
    };
    let mut app = AppState::new(num_tracks, num_steps);

    // Populate AppState with pattern steps and track names
    app.steps = if !patterns.is_empty() {
        patterns[0].steps.clone()
    } else {
        vec![vec![false; num_steps]; num_tracks]
    };
    
    // Initialize track names from loaded tracks
    app.track_names = tracks.iter().map(|t| t.name.clone()).collect();
    if app.track_names.is_empty() {
        app.track_names = (0..num_tracks).map(|i| format!("tr-{:<2}", i)).collect();
    }
    
    // Configure the app state with tracks and sample directory
    let sample_dir = project_path.join("samples");
    app = app.with_sample_dir(sample_dir).with_tracks(tracks);
    
    // Initialize the sequencer with the pattern and BPM, and connect to audio
    app.bpm = _project.bpm as u32;
    match app.initialize_sequencer(true) {
        Ok(_) => {
            info!("Sequencer and audio initialized successfully");
            
            // Configure audio effects based on metadata
            if !pattern_metas.is_empty() && app.audio.is_some() {
                if let Some(audio) = &app.audio {
                    if let Err(e) = audio.configure_effects(&pattern_metas) {
                        error!("Failed to configure audio effects: {}", e);
                    } else {
                        info!("Audio effects configured from pattern metadata");
                    }
                }
            }
        },
        Err(e) => {
            // Fall back to sequencer-only operation if audio fails
            error!("Failed to initialize audio: {}, continuing without audio", e);
            app.sequencer = Some(sequencer::Sequencer::new(app.bpm, app.steps.clone()));
        }
    }
    
    debug!("AppState initialized with {} tracks and {} steps", num_tracks, num_steps);

    // Ensure `terminal` is properly initialized
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    loop {
        terminal.draw(|f| {
            debug!("Drawing UI");
            let size = f.area();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Length(6),
                    Constraint::Length(4),
                ])
                .split(size);

            let status = if app.is_playing { "PLAYING" } else { "PAUSED" };
            let step_display = if app.is_playing { app.current_step + 1 } else { app.selected_step + 1 };
            
            let header = Paragraph::new(format!(
                "SONG: {} | BPM:{} STEP:{:02}/{} | {}", 
                _project.name, _project.bpm, step_display, num_steps, status
            )).block(Block::default().borders(Borders::ALL));
            f.render_widget(header, chunks[0]);

            use ratatui::widgets::Cell;

            let rows: Vec<Row> = app
                .steps
                .iter()
                .enumerate()
                .map(|(track_idx, steps)| {
                    // Check if this track has a trigger in the current step
                    let is_playing = app.is_playing && 
                        app.trigger_events.iter().any(|e| e.track_idx == track_idx);
                    
                    // Show visual feedback for playing tracks
                    let track_style = if is_playing {
                        Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD | Modifier::SLOW_BLINK)
                    } else {
                        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
                    };
                    
                    // Add track name and volume info
                    let volume_str = if let Some(vol) = app.get_track_volume(track_idx) {
                        format!("{:.1}", vol)
                    } else {
                        "?".to_string()
                    };

                    let mut cells: Vec<Cell> = vec![
                        Cell::from(format!("{} v{}", app.track_names[track_idx], volume_str))
                            .style(track_style),
                    ];
                    cells.extend(
                        steps
                            .iter()
                            .enumerate()
                            .map(|(i, &on)| {
                                let symbol = if on { "X" } else { "." };
                                let style = if app.is_playing && app.current_step == i {
                                    // Highlight current playing step
                                    Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
                                } else if app.selected_track == track_idx && app.selected_step == i {
                                    // Highlight selected cell
                                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                                } else {
                                    Style::default()
                                };
                                Cell::from(symbol).style(style)
                            })
                    );

                    Row::new(cells).height(1).bottom_margin(0)
                })
                .collect();
            let mut widths = vec![Constraint::Length(10)]; // Increased width for track names + volume
            widths.extend(std::iter::repeat(Constraint::Length(1)).take(16));
            let table = Table::new(rows, vec![Constraint::Length(1); 16])
                .block(Block::default().title("PATTERN VIEW").borders(Borders::ALL))
                .widths(widths);
            f.render_widget(table, chunks[1]);

            let footer = Paragraph::new("[Space] Toggle Step [P] Play/Pause [T] Test Sound [+/-] Volume [Arrows] Move [Q] Quit")
                .block(Block::default().borders(Borders::ALL));
            f.render_widget(footer, chunks[2]);
        })?;

        // Process sequencer events if it's playing
        if app.is_playing {
            app.process_sequencer_events();
        }
        
        if event::poll(std::time::Duration::from_millis(10))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Char(' ') => app.toggle_step(),
                    KeyCode::Char('p') => app.toggle_playback(),
                    KeyCode::Left => app.move_cursor_left(),
                    KeyCode::Right => app.move_cursor_right(),
                    KeyCode::Up => app.move_cursor_up(),
                    KeyCode::Down => app.move_cursor_down(),
                    KeyCode::Char('t') => {
                        // Test sound of the currently selected track
                        if let Err(e) = app.test_track_sound(app.selected_track) {
                            error!("Failed to test track sound: {}", e);
                        } else {
                            info!("Testing sound for track {}", app.selected_track);
                        }
                    },
                    KeyCode::Char('+') | KeyCode::Char('=') => {
                        // Increase volume of the currently selected track
                        if let Some(current_volume) = app.get_track_volume(app.selected_track) {
                            let new_volume = (current_volume + 0.1).min(1.0);
                            if let Err(e) = app.set_track_volume(app.selected_track, new_volume) {
                                error!("Failed to increase volume: {}", e);
                            }
                        }
                    },
                    KeyCode::Char('-') => {
                        // Decrease volume of the currently selected track
                        if let Some(current_volume) = app.get_track_volume(app.selected_track) {
                            let new_volume = (current_volume - 0.1).max(0.0);
                            if let Err(e) = app.set_track_volume(app.selected_track, new_volume) {
                                error!("Failed to decrease volume: {}", e);
                            }
                        }
                    },
                    _ => {}
                }
            }
        }
    }

    // Clean up audio resources
    info!("Shutting down application, cleaning up resources");
    app.cleanup_audio();
    
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    
    info!("Application shutdown complete");
    Ok(())
}
