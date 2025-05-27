
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, Paragraph, Row, Table},
    Terminal,
};
use std::io;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

struct AppState {
    steps: Vec<Vec<bool>>, // [track][step]
    selected_track: usize,
    selected_step: usize,
}

impl AppState {
    fn new(num_tracks: usize, num_steps: usize) -> Self {
        Self {
            steps: vec![vec![false; num_steps]; num_tracks],
            selected_track: 0,
            selected_step: 0,
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
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = AppState::new(4, 16);

    loop {
        terminal.draw(|f| {
            let size = f.area();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Length(6),
                    Constraint::Length(4),
                ])
                .split(size);

            let header = Paragraph::new("SONG 001 | BPM:120 STEP:00/16 | PAUSED")
                .block(Block::default().borders(Borders::ALL));
            f.render_widget(header, chunks[0]);

            use ratatui::widgets::Cell;

            let rows: Vec<Row> = app
                .steps
                .iter()
                .enumerate()
                .map(|(track_idx, steps)| {
                    let mut cells: Vec<Cell> = vec![
                        Cell::from(format!("tr-{:<2}", track_idx))
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
