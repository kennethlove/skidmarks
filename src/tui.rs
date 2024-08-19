use crate::cli::get_database_url;
use crate::db::Database;
use crate::sorting::{SortByDirection, SortByField};
use dioxus::html::ScrollData;
use ratatui::widgets::{
    Block, BorderType, Borders, Cell, HighlightSpacing, Paragraph, Row, Scrollbar,
    ScrollbarOrientation, ScrollbarState, Table, TableState, Tabs,
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    crossterm::{
        event::{self, DisableMouseCapture, EnableMouseCapture, KeyCode, KeyEventKind},
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    },
    layout::{Constraint, Layout, Rect},
    prelude::*,
    text::Text,
    Terminal,
};
use std::io;
use term_size::dimensions;

enum InputMode {
    Normal,
    Insert,
}

struct App {
    input_mode: InputMode,
    table_state: TableState,
    scrollbar_state: ScrollbarState,
    db: Database,
}

impl App {
    pub fn new() -> Self {
        let db = Database::new(&get_database_url()).unwrap();
        App {
            input_mode: InputMode::Normal,
            table_state: TableState::default().with_selected(0),
            scrollbar_state: ScrollbarState::new(db.num_tasks()).position(0),
            db,
        }
    }

    pub fn select_down(&mut self) {
        let i = match self.table_state.selected() {
            Some(i) => {
                if i < self.db.num_tasks().saturating_sub(1) {
                    i + 1
                } else {
                    0
                }
            }
            None => 0,
        };
        self.table_state.select(Some(i));
        self.scrollbar_state = self.scrollbar_state.position(i * 2);
    }

    pub fn select_up(&mut self) {
        let i = match self.table_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.db.num_tasks().saturating_sub(1)
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.table_state.select(Some(i));
        self.scrollbar_state = self.scrollbar_state.position(i);
    }
}

pub fn main() -> io::Result<()> {
    enable_raw_mode()?;
    execute!(io::stdout(), EnterAlternateScreen, EnableMouseCapture)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;
    terminal.clear()?;

    let mut app = App::new();
    let res = run_app(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{err:?}");
    }
    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: &mut App) -> io::Result<()> {
    loop {
        // Draw the UI
        terminal.draw(|frame| {
            let _ = layout_app(&mut app, frame);
        })?;

        // Handle events
        if event::poll(std::time::Duration::from_millis(16))? {
            if let event::Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Char('j') => app.select_down(),
                        KeyCode::Char('k') => app.select_up(),
                        _ => {}
                    }
                }
            }
        }
    }
    Ok(())
}

/// helper function to create a centered rect using up certain percentage of the available rect `r`
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    // Cut the given rectangle into three vertical pieces
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    // Then cut the middle vertical piece into three width-wise pieces
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1] // Return the middle chunk
}

/// Create the outermost layout and call functions to draw the header, main, and footer
fn layout_app(app: &mut App, frame: &mut Frame) -> io::Result<()> {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![
            Constraint::Length(2), // header
            Constraint::Fill(1),   // main
            Constraint::Length(2), // footer
        ])
        .split(frame.area());

    draw_header(frame, chunks[0])?;

    layout_main(app, frame, chunks[1])?;

    draw_footer(frame, chunks[2])?;

    Ok(())
}

fn draw_header(frame: &mut Frame, area: Rect) -> io::Result<()> {
    let block = Block::new()
        .borders(Borders::BOTTOM)
        .border_type(BorderType::Thick);
    let text = "Skidmarks";
    let paragraph = Paragraph::new(text)
        .alignment(Alignment::Center)
        .block(block);
    frame.render_widget(paragraph, area);
    Ok(())
}

fn draw_footer(frame: &mut Frame, area: Rect) -> io::Result<()> {
    let block = Block::new()
        .borders(Borders::TOP)
        .border_type(BorderType::Thick);
    let text = "Press 'q' to quit";
    let paragraph = Paragraph::new(text)
        .alignment(Alignment::Center)
        .block(block);
    frame.render_widget(paragraph, area);
    Ok(())
}

fn layout_main(app: &mut App, frame: &mut Frame, area: Rect) -> io::Result<()> {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Fill(1),
            Constraint::Length(4),
        ])
        .split(area);

    draw_tabs(frame, chunks[0])?;

    layout_content(app, frame, chunks[1])?;

    draw_form(frame, chunks[2])?;

    Ok(())
}

fn draw_tabs(frame: &mut Frame, area: Rect) -> io::Result<()> {
    let tabs = Tabs::new(vec!["All", "Waiting", "Missed", "Completed"])
        .block(Block::default().borders(Borders::BOTTOM))
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().fg(Color::Yellow))
        .select(0)
        .divider(symbols::DOT);
    frame.render_widget(tabs, area);
    Ok(())
}

fn draw_form(frame: &mut Frame, area: Rect) -> io::Result<()> {
    let form_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(66), Constraint::Percentage(33)])
        .split(area);

    let task_block = Block::default().borders(Borders::ALL).title("Task");
    let task = Paragraph::new("Task goes here")
        .block(task_block)
        .alignment(Alignment::Left);
    frame.render_widget(task, form_layout[0]);

    let freq_block = Block::default().borders(Borders::ALL).title("Frequency");
    let freq = Paragraph::new("Daily")
        .block(freq_block)
        .alignment(Alignment::Left);
    frame.render_widget(freq, form_layout[1]);

    Ok(())
}

fn layout_content(app: &mut App, frame: &mut Frame, area: Rect) -> io::Result<()> {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Fill(1), Constraint::Length(1)])
        .split(area);

    draw_table(app, frame, chunks[0])?;

    let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
        .begin_symbol(Some("▲"))
        .end_symbol(Some("▼"));
    frame.render_stateful_widget(scrollbar, chunks[1], &mut app.scrollbar_state);
    Ok(())
}

fn draw_table(app: &mut App, frame: &mut Frame, area: Rect) -> io::Result<()> {
    let widths = [
        Constraint::Fill(1),    // Task
        Constraint::Length(7),  // Freq
        Constraint::Length(3),  // Status
        Constraint::Length(10), // Last Checkin
        Constraint::Length(7),  // Current Streak
        Constraint::Length(7),  // Longest Streak
        Constraint::Length(7),  // Total Checkins
    ];

    let rows = get_rows();

    let header_style = Style::default().fg(Color::Green);

    let table = Table::new(rows, widths)
        .column_spacing(1)
        .header(
            Row::new(vec![
                Cell::from("\nTask"),
                Cell::from("\nFreq"),
                Cell::from(Text::from("\nStatus").alignment(Alignment::Center)),
                Cell::from("Last\nCheck In"),
                Cell::from("Current\nStreak"),
                Cell::from("Longest\nStreak"),
                Cell::from("Total\nCheckins"),
            ])
            .style(header_style)
            .height(2),
        )
        .footer(Row::new(vec!["Total Tasks: 10"]))
        .bg(Color::Black)
        .highlight_spacing(HighlightSpacing::WhenSelected)
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().bg(Color::White).fg(Color::Black));

    frame.render_stateful_widget(table, area, &mut app.table_state);

    Ok(())
}

fn get_rows() -> Vec<Row<'static>> {
    let database = Database::new(&get_database_url());
    let streaks = database
        .unwrap()
        .get_sorted(SortByField::Task, SortByDirection::Ascending);

    let mut rows = vec![];
    let (w, _) = dimensions().unwrap();
    let w = w.saturating_sub(50);
    let mut h = 1;

    for streak in streaks {
        let task_lines = textwrap::wrap(&streak.task, w);
        h = task_lines.len();
        let task = task_lines.join("\n");

        let freq = streak.frequency.to_string();
        let status = streak.emoji_status().to_string();
        let status = Text::from(status).alignment(Alignment::Center);
        let last_checkin = streak
            .last_checkin
            .map(|dt| dt.format("%Y-%m-%d").to_string())
            .unwrap_or("None".to_string());
        let current_streak =
            Text::from(streak.current_streak.to_string()).alignment(Alignment::Center);
        let longest_streak =
            Text::from(streak.longest_streak.to_string()).alignment(Alignment::Center);
        let total_checkins =
            Text::from(streak.total_checkins.to_string()).alignment(Alignment::Center);

        let row = Row::new(vec![
            Cell::from(task.clone()),
            Cell::from(freq),
            Cell::from(status),
            Cell::from(last_checkin),
            Cell::from(current_streak),
            Cell::from(longest_streak),
            Cell::from(total_checkins),
        ])
        .height(h as u16);
        rows.push(row.clone());
    }
    rows
}
