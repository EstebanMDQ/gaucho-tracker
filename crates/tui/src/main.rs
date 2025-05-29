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

struct AppState {
    steps: Vec<Vec<bool>>,
    selected_track: usize,
    selected_step: usize,
    track_names: Vec<String>,
}

impl AppState {
    fn new(num_tracks: usize, num_steps: usize) -> Self {
        Self {
            steps: vec![vec![false; num_steps]; num_tracks],
            selected_track: 0,
            selected_step: 0,
            track_names: vec![],
        }
    }

    fn toggle_step(&mut self) {
        let val = &mut self.steps[self.selected_track][self.selected_step];
        *val = !*val;
    }

    fn move_cursor(&mut self, direction: KeyCode) {
        match direction {
            KeyCode::Left => {
                if self.selected_step > 0 {
                    self.selected_step -= 1;
                }
            }
            KeyCode::Right => {
                if self.selected_step + 1 < self.steps[0].len() {
                    self.selected_step += 1;
                }
            }
            KeyCode::Up => {
                if self.selected_track > 0 {
                    self.selected_track -= 1;
                }
            }
            KeyCode::Down => {
                if self.selected_track + 1 < self.steps.len() {
                    self.selected_track += 1;
                }
            }
            _ => {}
        }
    }
}

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
    println!("Resolved project path: {:?}", project_path);
    
    let (_project, tracks, patterns) = match load_project(project_path) {
        Ok((proj, trks, pats)) => {
            info!("Project loaded successfully: {}", proj.name);
            (proj, trks, pats)
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

            let header = Paragraph::new(format!(
                "SONG: {} | BPM:{} STEP:{:02}/{} | PAUSED", 
                _project.name, _project.bpm, app.selected_step + 1, num_steps
            )).block(Block::default().borders(Borders::ALL));
            f.render_widget(header, chunks[0]);

            use ratatui::widgets::Cell;

            let rows: Vec<Row> = app
                .steps
                .iter()
                .enumerate()
                .map(|(track_idx, steps)| {
                    let mut cells: Vec<Cell> = vec![
                        Cell::from(app.track_names[track_idx].clone())
                            .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                    ];
                    cells.extend(
                        steps
                            .iter()
                            .enumerate()
                            .map(|(i, &on)| {
                                let symbol = if on { "X" } else { "." };
                                let style = if app.selected_track == track_idx && app.selected_step == i {
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
            let mut widths = vec![Constraint::Length(6)];
            widths.extend(std::iter::repeat(Constraint::Length(1)).take(16));
            let table = Table::new(rows, vec![Constraint::Length(1); 16])
                .block(Block::default().title("PATTERN VIEW").borders(Borders::ALL))
                .widths(widths);
            f.render_widget(table, chunks[1]);

            let footer = Paragraph::new("[Space] Toggle [Arrows] Move [Q] Quit")
                .block(Block::default().borders(Borders::ALL));
            f.render_widget(footer, chunks[2]);
        })?;

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Char(' ') => app.toggle_step(),
                    k @ KeyCode::Left
                    | k @ KeyCode::Right
                    | k @ KeyCode::Up
                    | k @ KeyCode::Down => app.move_cursor(k),
                    _ => {}
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}
