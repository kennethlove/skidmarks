use crate::cli::get_database_url;
use crate::color::TuiStyles;
use crate::db::Database;
use crate::filtering::{filter_by_status, FilterByStatus};
use crate::sorting::{SortByDirection, SortByField};
use crate::streak::{Frequency, Streak};
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

#[derive(Clone, Debug)]
struct NewStreak {
    task: String,
    frequency: Frequency,
}

impl Default for NewStreak {
    fn default() -> Self {
        NewStreak {
            task: String::default(),
            frequency: Frequency::Daily,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
enum AppState {
    Normal,
    Insert,
    Search,
    Delete,
}

#[derive(Clone, Debug)]
struct App {
    app_state: AppState,
    table_state: TableState,
    scrollbar_state: ScrollbarState,
    db: Database,
    sort_by_field: SortByField,
    sort_by_direction: SortByDirection,
    filter_by_status: FilterByStatus,
    tab_state: u8,
    search_phrase: String,
    new_streak: NewStreak,
    styles: TuiStyles,
}

impl App {
    pub fn new() -> Self {
        let db = Database::new(&get_database_url()).unwrap();
        App {
            app_state: AppState::Normal,
            table_state: TableState::default().with_selected(0),
            scrollbar_state: ScrollbarState::new(db.num_tasks()).position(0),
            db,
            sort_by_field: SortByField::Task,
            sort_by_direction: SortByDirection::Ascending,
            filter_by_status: FilterByStatus::All,
            tab_state: 0,
            search_phrase: String::default(),
            new_streak: NewStreak::default(),
            styles: TuiStyles::new(),
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

    pub fn check_in(&mut self) -> io::Result<()> {
        let i = self.table_state.selected().unwrap();
        let mut streak = self
            .db
            .get_by_index(
                i,
                self.sort_by_field.clone(),
                self.sort_by_direction.clone(),
                self.filter_by_status.clone(),
            )
            .unwrap();
        streak.checkin();
        self.db.update(streak.id, streak)?;
        self.db.save()?;
        Ok(())
    }

    pub fn add_streak(&mut self) -> io::Result<()> {
        let streak = match self.new_streak.frequency {
            Frequency::Daily => Streak::new_daily(self.new_streak.task.clone()),
            Frequency::Weekly => Streak::new_weekly(self.new_streak.task.clone()),
        };
        self.db.add(streak)?;
        self.db.save()?;
        Ok(())
    }

    pub fn delete_selected(&mut self) -> io::Result<()> {
        let i = self.table_state.selected().unwrap();
        let streak = self
            .db
            .get_by_index(
                i,
                self.sort_by_field.clone(),
                self.sort_by_direction.clone(),
                self.filter_by_status.clone(),
            )
            .unwrap();
        self.db.delete(streak.id)?;
        self.db.save()?;
        Ok(())
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
                    match app.app_state {
                        AppState::Normal => match key.code {
                            KeyCode::Char('q') => break,
                            KeyCode::Char('j') => app.select_down(),
                            KeyCode::Char('k') => app.select_up(),
                            KeyCode::Char('c') => app.check_in()?,
                            KeyCode::Char('z') => match app.sort_by_direction {
                                SortByDirection::Ascending => {
                                    app.sort_by_direction = SortByDirection::Descending
                                }
                                SortByDirection::Descending => {
                                    app.sort_by_direction = SortByDirection::Ascending
                                }
                            },
                            KeyCode::Char('f') => match app.filter_by_status {
                                FilterByStatus::All => {
                                    app.tab_state = 1;
                                    app.filter_by_status = FilterByStatus::Waiting
                                }
                                FilterByStatus::Waiting => {
                                    app.tab_state = 2;
                                    app.filter_by_status = FilterByStatus::Missed
                                }
                                FilterByStatus::Missed => {
                                    app.tab_state = 3;
                                    app.filter_by_status = FilterByStatus::Done
                                }
                                FilterByStatus::Done => {
                                    app.tab_state = 0;
                                    app.filter_by_status = FilterByStatus::All
                                }
                            },
                            KeyCode::Char('o') => match app.sort_by_field {
                                SortByField::Task => app.sort_by_field = SortByField::Frequency,
                                SortByField::Frequency => app.sort_by_field = SortByField::Status,
                                SortByField::Status => app.sort_by_field = SortByField::LastCheckIn,
                                SortByField::LastCheckIn => {
                                    app.sort_by_field = SortByField::CurrentStreak
                                }
                                SortByField::CurrentStreak => {
                                    app.sort_by_field = SortByField::LongestStreak
                                }
                                SortByField::LongestStreak => {
                                    app.sort_by_field = SortByField::TotalCheckins
                                }
                                SortByField::TotalCheckins => app.sort_by_field = SortByField::Task,
                            },
                            KeyCode::Char('s') => {
                                app.search_phrase = "".to_string();
                                app.app_state = AppState::Search;
                            }
                            KeyCode::Char('a') => {
                                app.new_streak = NewStreak::default();
                                app.app_state = AppState::Insert;
                            }
                            KeyCode::Char('d') => {
                                app.app_state = AppState::Delete;
                            }
                            _ => {}
                        },
                        AppState::Insert => match key.code {
                            KeyCode::Esc => app.app_state = AppState::Normal,
                            KeyCode::Enter => {
                                app.add_streak()?;
                                app.app_state = AppState::Normal;
                            }
                            KeyCode::Backspace => {
                                app.new_streak.task.pop();
                            }
                            KeyCode::Char(c) => {
                                app.new_streak.task.push(c);
                            }
                            KeyCode::Tab => match app.new_streak.frequency {
                                Frequency::Daily => app.new_streak.frequency = Frequency::Weekly,
                                Frequency::Weekly => app.new_streak.frequency = Frequency::Daily,
                            },
                            _ => {}
                        },
                        AppState::Search => match key.code {
                            KeyCode::Esc => {
                                app.search_phrase = "".to_string();
                                app.app_state = AppState::Normal
                            }
                            KeyCode::Enter => app.app_state = AppState::Normal,
                            KeyCode::Backspace => {
                                app.search_phrase.pop();
                            }
                            KeyCode::Char(c) => {
                                app.search_phrase.push(c);
                            }
                            _ => {}
                        },
                        AppState::Delete => match key.code {
                            KeyCode::Char('y') => {
                                app.delete_selected()?;
                                app.app_state = AppState::Normal;
                            }
                            KeyCode::Char('n') => {
                                app.app_state = AppState::Normal;
                            }
                            _ => {}
                        },
                    }
                }
            }
        }
    }
    Ok(())
}

/// Create the outermost layout and call functions to draw the header, main, and footer
fn layout_app(app: &mut App, frame: &mut Frame) -> io::Result<()> {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![
            Constraint::Length(2), // header
            Constraint::Fill(1),   // main
            Constraint::Length(3), // footer
        ])
        .split(frame.area());

    draw_header(app, frame, chunks[0])?;

    match app.app_state {
        AppState::Search => layout_search(app, frame, chunks[1])?,
        AppState::Insert => layout_add(app, frame, chunks[1])?,
        AppState::Delete => layout_delete(app, frame, chunks[1])?,
        _ => layout_main(app, frame, chunks[1])?,
    }

    draw_footer(app, frame, chunks[2])?;

    Ok(())
}

fn draw_header(app: &mut App, frame: &mut Frame, area: Rect) -> io::Result<()> {
    let block = Block::new()
        .borders(Borders::BOTTOM)
        .border_type(BorderType::Thick)
        .bg(app.styles.background);
    let text = "Skidmarks";
    let paragraph = Paragraph::new(text)
        .alignment(Alignment::Center)
        .block(block);
    frame.render_widget(paragraph, area);
    Ok(())
}

fn draw_footer(app: &mut App, frame: &mut Frame, area: Rect) -> io::Result<()> {
    let block = Block::new()
        .bg(app.styles.background)
        .borders(Borders::TOP)
        .border_type(BorderType::Thick);
    let text = match app.app_state {
        AppState::Normal => "[f] filter, [o] change sort order, [z] reverse order, [s] search\n[j/k] select, [c] check in, [a] add, [d] delete, [q] quit",
        AppState::Insert => "[Esc] cancel, [Enter] save, [Tab] toggle frequency",
        AppState::Search => "[Esc] cancel, [Enter] search, [Backspace] delete",
        AppState::Delete => "[y] yes, [n] no",
    };
    let help_text = Paragraph::new(text)
        .alignment(Alignment::Center)
        .block(block);
    frame.render_widget(help_text, area);
    Ok(())
}

fn layout_main(app: &mut App, frame: &mut Frame, area: Rect) -> io::Result<()> {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Fill(1)])
        .split(area);

    draw_tabs(app, frame, chunks[0])?;

    layout_content(app, frame, chunks[1])?;

    Ok(())
}

fn draw_tabs(app: &mut App, frame: &mut Frame, area: Rect) -> io::Result<()> {
    let tabs = Tabs::new(vec!["All", "Waiting", "Missed", "Completed"])
        .block(
            Block::default()
                .bg(app.styles.background)
                .borders(Borders::BOTTOM)
                .title_alignment(Alignment::Left)
                .title("Filter"),
        )
        .style(Style::default().fg(app.styles.tab_fg))
        .highlight_style(Style::default().fg(app.styles.selected_tab_fg))
        .select(app.tab_state.into())
        .divider(symbols::DOT);
    frame.render_widget(tabs, area);
    Ok(())
}

fn layout_content(app: &mut App, frame: &mut Frame, area: Rect) -> io::Result<()> {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Fill(1), Constraint::Length(2)])
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

    let rows = get_rows(app);

    let header_style = Style::default().add_modifier(Modifier::BOLD);
    let sorted_by_style = Style::default().fg(app.styles.selected_tab_fg);
    let sorted_icon = match app.sort_by_direction {
        SortByDirection::Ascending => "⬆",
        SortByDirection::Descending => "⬇",
    };
    let header_pairs = vec![
        ("\nTask", SortByField::Task),
        ("\nFreq.", SortByField::Frequency),
        ("\nStatus", SortByField::Status),
        ("Last\nCheckin", SortByField::LastCheckIn),
        ("Current\nStreak", SortByField::CurrentStreak),
        ("Longest\nStreak", SortByField::LongestStreak),
        ("Total\nCheckins", SortByField::TotalCheckins),
    ];
    let header_row = Row::new(
        header_pairs
            .iter()
            .map(|(name, field)| {
                let style = if *field == app.sort_by_field {
                    sorted_by_style
                } else {
                    header_style
                };
                let text = if *field == app.sort_by_field {
                    format!("{} {}", name, sorted_icon)
                } else {
                    name.to_string()
                };
                Cell::from(text).style(style)
            })
            .collect::<Vec<Cell>>(),
    );

    let table = Table::new(rows.clone(), widths)
        .column_spacing(1)
        .header(header_row.style(header_style).height(2))
        .footer(Row::new(vec![
            if app.search_phrase.is_empty() {
                "".to_string()
            } else {
                format!("Search: {}", app.search_phrase)
            },
            "".to_string(),
            "".to_string(),
            "".to_string(),
            "".to_string(),
            "".to_string(),
            format!("{}/{}", rows.clone().len(), app.db.num_tasks()),
        ]))
        .bg(app.styles.row_bg)
        .style(Style::default().fg(app.styles.row_fg))
        .highlight_spacing(HighlightSpacing::WhenSelected)
        .highlight_style(
            Style::default()
                .bg(app.styles.highlight_bg)
                .fg(app.styles.highlight_fg),
        )
        .block(Block::default().bg(app.styles.background));

    frame.render_stateful_widget(table, area, &mut app.table_state);

    Ok(())
}

fn get_rows(app: &mut App) -> Vec<Row<'static>> {
    let app = app.clone();
    let database = Database::new(&get_database_url());
    let streaks = database
        .unwrap()
        .get_sorted(app.sort_by_field, app.sort_by_direction);
    let mut streaks = filter_by_status(streaks, app.filter_by_status);
    if !app.search_phrase.is_empty() {
        streaks = streaks
            .into_iter()
            .filter(|streak| {
                streak
                    .task
                    .to_lowercase()
                    .contains(&app.search_phrase.to_lowercase())
            })
            .collect();
    }

    let styles = TuiStyles::new();

    let mut rows = vec![];
    let (w, _) = dimensions().unwrap();
    let w = w.saturating_sub(50);

    for (i, streak) in streaks.iter().enumerate() {
        let task_lines = textwrap::wrap(&streak.task, w);
        let h = task_lines.len();
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

        let row_style = if i % 2 == 0 {
            Style::default().fg(styles.row_fg).bg(styles.row_bg)
        } else {
            Style::default().fg(styles.alt_row_fg).bg(styles.alt_row_bg)
        };

        let row = Row::new(vec![
            Cell::from(task.clone()),
            Cell::from(freq),
            Cell::from(status),
            Cell::from(last_checkin),
            Cell::from(current_streak),
            Cell::from(longest_streak),
            Cell::from(total_checkins),
        ])
        .style(row_style)
        .height(h as u16);
        rows.push(row.clone());
    }
    rows
}

fn layout_search(app: &mut App, frame: &mut Frame, area: Rect) -> io::Result<()> {
    let layout = get_centered_layout(3).split(area);
    draw_search(app, frame, layout[1])?;

    Ok(())
}

fn draw_search(app: &mut App, frame: &mut Frame, area: Rect) -> io::Result<()> {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(app.styles.highlight_bg))
        .title("Search")
        .title_alignment(Alignment::Center);
    let paragraph = Paragraph::new(app.search_phrase.clone())
        .style(Style::default().fg(app.styles.foreground))
        .block(block)
        .alignment(Alignment::Left);
    frame.render_widget(paragraph, area);
    frame.set_cursor_position((area.x + 1 + app.search_phrase.len() as u16, area.y + 1));
    Ok(())
}

fn layout_add(app: &mut App, frame: &mut Frame, area: Rect) -> io::Result<()> {
    let layout = get_centered_layout(6).split(area);
    draw_add(app, frame, layout[1])?;

    Ok(())
}

fn draw_add(app: &mut App, frame: &mut Frame, area: Rect) -> io::Result<()> {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Length(3)])
        .split(area);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(app.styles.highlight_bg))
        .title("New Streak")
        .title_alignment(Alignment::Center);
    let task = Paragraph::new(app.new_streak.task.clone())
        .style(Style::default().fg(app.styles.foreground))
        .block(block)
        .alignment(Alignment::Left);
    frame.render_widget(task, layout[0]);
    frame.set_cursor_position((
        layout[0].x + 1 + app.new_streak.task.len() as u16,
        layout[0].y + 1,
    ));
    frame.render_widget(draw_add_tabs(app), layout[1]);
    Ok(())
}

fn draw_add_tabs(app: &mut App) -> Tabs {
    let select = match app.new_streak.frequency {
        Frequency::Daily => 0,
        Frequency::Weekly => 1,
    };
    let tabs = Tabs::new(vec!["Daily", "Weekly"])
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title_alignment(Alignment::Center)
                .title("Frequency"),
        )
        .style(Style::default().fg(app.styles.tab_fg))
        .highlight_style(Style::default().fg(app.styles.selected_tab_fg))
        .select(select)
        .divider(symbols::DOT);
    tabs
}

fn get_centered_layout(height: u16) -> Layout {
    Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Fill(1),
            Constraint::Length(height),
            Constraint::Fill(1),
        ])
}
fn layout_delete(app: &mut App, frame: &mut Frame, area: Rect) -> io::Result<()> {
    let layout = get_centered_layout(3).split(area);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(
            Style::default()
                .fg(app.styles.background)
                .bg(app.styles.danger),
        )
        .title("Delete")
        .title_alignment(Alignment::Center)
        .fg(app.styles.background)
        .bg(app.styles.danger);
    let text = "Are you sure you want to delete this streak?";
    let paragraph = Paragraph::new(text)
        .block(block)
        .alignment(Alignment::Center);
    frame.render_widget(paragraph, layout[1]);

    Ok(())
}
