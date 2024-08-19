use std::io;

use ratatui::widgets::{
    Block, BorderType, Borders, Row, Scrollbar, ScrollbarOrientation, ScrollbarState, TableState,
    Tabs,
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
    widgets::{Paragraph, Table},
    Terminal,
};

struct App {}

impl App {
    pub fn new() -> Self {
        App {}
    }
}

pub fn main() -> io::Result<()> {
    enable_raw_mode()?;
    execute!(io::stdout(), EnterAlternateScreen, EnableMouseCapture)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;
    terminal.clear()?;

    let app = App::new();
    let res = run_app(&mut terminal, app);

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

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: App) -> io::Result<()> {
    loop {
        // Draw the UI
        terminal.draw(|frame| {
            let _ = layout_app(frame);
        })?;

        // Handle events
        if event::poll(std::time::Duration::from_millis(16))? {
            if let event::Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('q') {
                    break;
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
fn layout_app(frame: &mut Frame) -> io::Result<()> {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![
            Constraint::Length(2), // header
            Constraint::Fill(1),   // main
            Constraint::Length(2), // footer
        ])
        .split(frame.area());

    draw_header(frame, chunks[0])?;

    layout_main(frame, chunks[1])?;

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

fn layout_main(frame: &mut Frame, area: Rect) -> io::Result<()> {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Fill(1),
            Constraint::Length(4),
        ])
        .split(area);

    draw_tabs(frame, chunks[0])?;

    layout_content(frame, chunks[1])?;

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

fn layout_content(frame: &mut Frame, area: Rect) -> io::Result<()> {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Fill(1), Constraint::Length(1)])
        .split(area);

    draw_table(frame, chunks[0])?;

    let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
        .begin_symbol(Some("▲"))
        .end_symbol(Some("▼"));
    let mut scrollbar_state = ScrollbarState::new(1).position(0);
    frame.render_stateful_widget(scrollbar, chunks[1], &mut scrollbar_state);
    Ok(())
}

fn draw_table(frame: &mut Frame, area: Rect) -> io::Result<()> {
    let mut table_state = TableState::default();
    let widths = [
        Constraint::Length(20),
        Constraint::Length(10),
        Constraint::Length(10),
        Constraint::Length(10),
        Constraint::Length(10),
        Constraint::Length(10),
        Constraint::Length(10),
    ];
    let rows = [
        Row::new(vec![
            "Task",
            "Freq",
            "Status",
            "Last Check In",
            "Current Streak",
            "Longest Streak",
            "Total",
        ]),
        Row::new(vec![
            "Task",
            "Freq",
            "Status",
            "Last Check In",
            "Current Streak",
            "Longest Streak",
            "Total",
        ]),
        Row::new(vec![
            "Task",
            "Freq",
            "Status",
            "Last Check In",
            "Current Streak",
            "Longest Streak",
            "Total",
        ]),
    ];
    let table = Table::new(rows, widths)
        .column_spacing(1)
        .header(Row::new(vec![
            "Task",
            "Freq",
            "Status",
            "Last Check In",
            "Current Streak",
            "Longest Streak",
            "Total",
        ]))
        .footer(Row::new(vec!["Total Tasks: 10"]))
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().bold().reversed());

    frame.render_stateful_widget(table, area, &mut table_state);

    Ok(())
}
