use std::io::{self, stdout};
use crate::db::Database;
use crate::cli::get_database_url;

use ratatui::{
    crossterm::{
        event::{self, Event, KeyCode},
        terminal::{
            disable_raw_mode, enable_raw_mode,
            EnterAlternateScreen, LeaveAlternateScreen,
        },
        ExecutableCommand,
    },
    prelude::*,
    widgets::*,
};

pub fn main() -> io::Result<()> {
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    let mut should_quit = false;
    while !should_quit {
        terminal.draw(ui)?;
        should_quit = handle_events()?;
    }

    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}

fn handle_events() -> io::Result<bool> {
    if event::poll(std::time::Duration::from_millis(50))? {
        if let Event::Key(key) = event::read()? {
            if key.kind == event::KeyEventKind::Press && key.code == KeyCode::Char('q') {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

fn ui(frame: &mut Frame) {
    let db_url = get_database_url();
    let database = Database::new(&db_url.as_str()).expect("Failed to load database");
    frame.render_widget(
        Paragraph::new("Hello World").block(Block::bordered().title("Skidmarks")),
        frame.size(),
    );
    for streak in database.get_all().unwrap() {
        frame.render_widget(
            Paragraph::new(streak.task).block(Block::bordered()), frame.size()
        );
    }
}
