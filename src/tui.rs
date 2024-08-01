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
    let mut database = Database::new(&db_url.as_str()).expect("Failed to load database");

    let main_layout = Layout::new(
        Direction::Vertical,
        [
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(1),
        ],
    )
    .split(frame.size());

    frame.render_widget(
        Block::new().borders(Borders::TOP).title("Title Bar"),
        main_layout[0],
    );
    frame.render_widget(
        Block::new().borders(Borders::TOP).title("Status Bar"),
        main_layout[2],
    );

    let inner_layout = Layout::new(
        Direction::Horizontal,
        [
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ],
    ).split(main_layout[1]);


    match database.get_all() {
        Some(streaks) => {
            let mut rows = vec![];
            for streak in streaks {
                let row = Row::new(vec![
                    Cell::from(streak.task.clone()),
                    Cell::from(streak.frequency.to_string()),
                    Cell::from(streak.status().to_string()),
                ]);
                rows.push(row);
            }

            let widths = [
                Constraint::Length(50),
                Constraint::Length(10),
                Constraint::Length(10),
            ];

            let table = Table::new(rows, widths)
                .header(
                    Row::new(vec!["Task", "Frequency", "Status"])
                        .style(Style::default().fg(Color::Yellow))
                )
                .highlight_style(Style::new().reversed())
                .highlight_symbol(">>");

            frame.render_widget(table, inner_layout[0]);
        }
        None => {
            let error = Text::raw("Failed to load streaks");
            frame.render_widget(error, inner_layout[0]);
        }
    }
}
