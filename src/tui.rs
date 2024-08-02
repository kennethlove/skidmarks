use std::io;
use crate::db::Database;
use crate::cli::get_database_url;
use crate::streaks::Streak;
use style::palette::tailwind;
use ratatui::{
    backend::{Backend, CrosstermBackend},
    crossterm::{
        event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    },
    layout::{Constraint, Layout, Margin, Rect},
    prelude::*,
    style::{self, Color, Modifier, Style, Stylize},
    terminal::{Frame, Terminal},
    text::{Line, Span, Text},
    widgets::{
        Block, Borders, BorderType, Cell, HighlightSpacing, Paragraph, Row, Scrollbar, ScrollbarOrientation,
        ScrollbarState, Table, TableState,
    },
};
use tui_confirm_dialog::{ButtonLabel, ConfirmDialog, ConfirmDialogState, Listener};
use unicode_width::UnicodeWidthStr;


const PALETTES: [tailwind::Palette; 4] = [
    tailwind::BLUE,
    tailwind::EMERALD,
    tailwind::INDIGO,
    tailwind::RED,
];

const INFO_TEXT: &str = "[Q]uit | [↑] [↓] Select | [C]heck in | [A]dd | [R]emove";
const ITEM_HEIGHT: usize = 4;


struct TableColors {
    buffer_bg: Color,
    header_bg: Color,
    header_fg: Color,
    row_fg: Color,
    selected_style_fg: Color,
    normal_row_color: Color,
    alt_row_color: Color,
    footer_border_color: Color,
}

impl TableColors {
    const fn new(color: &tailwind::Palette) -> Self {
        Self {
            buffer_bg: tailwind::SLATE.c950,
            header_bg: color.c900,
            header_fg: tailwind::SLATE.c200,
            row_fg: tailwind::SLATE.c200,
            selected_style_fg: color.c400,
            normal_row_color: tailwind::SLATE.c950,
            alt_row_color: tailwind::SLATE.c900,
            footer_border_color: color.c400,
        }
    }
}

struct App {
    state: TableState,
    items: Vec<Data>,
    longest_item_lens: [usize; 5],
    scroll_state: ScrollbarState,
    colors: TableColors,
    db: Database,
    remove_popup: ConfirmDialogState,
    create_popup: ConfirmDialogState,
    popup_tx: std::sync::mpsc::Sender<Listener>,
    popup_rx: std::sync::mpsc::Receiver<Listener>,
}

impl App {
    fn new() -> Self {
        let (tx, rx) = std::sync::mpsc::channel();

        let mut db = Database::new(&get_database_url()).expect("Failed to load database");
        let data_vec: Vec<Data> = db.get_all()
            .unwrap_or_default()
            .into_iter()
            .map(Data::from)
            .collect();

        Self {
            state: TableState::default().with_selected(0),
            items: data_vec.clone(),
            longest_item_lens: constraint_len_calculator(&data_vec).into(),
            scroll_state: ScrollbarState::default(),
            colors: TableColors::new(&PALETTES[1]),
            db,
            remove_popup: ConfirmDialogState::default(),
            create_popup: ConfirmDialogState::default(),
            popup_tx: tx,
            popup_rx: rx,
        }
    }

    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 { 0 }
                else { i + 1 }
            },
            None => 0,
        };
        self.state.select(Some(i));
        self.scroll_state = self.scroll_state.position(i * ITEM_HEIGHT);
    }

    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 { self.items.len() - 1 }
                else { i - 1 }
            },
            None => 0,
        };
        self.state.select(Some(i));
        self.scroll_state = self.scroll_state.position(i * ITEM_HEIGHT);
    }

    pub fn check_in(&mut self) {
        let selected = self.state.selected().unwrap();
        let mut streak = self.db.get_one(selected as u32).unwrap();
        streak.checkin();
        let _ = self.db.update(selected as u32, &streak);
        let _ = self.db.save();
        self.items[selected] = Data::from(streak);
    }

    pub fn remove(&mut self) {
        let selected = self.state.selected().unwrap();

        let _ = self.db.delete(selected as u32);
        let _ = self.db.save();
        self.items.remove(selected);
    }
}

#[derive(Debug, Clone)]
struct Data {
    task: String,
    frequency: String,
    emoji: String,
    last_checkin: String,
    total_checkins: String,
}

impl From<Streak> for Data {
    fn from(value: Streak) -> Self {
        Self::new(value)
    }
}

impl Data {
    const fn ref_array(&self) -> [&String; 5] {
        [
            &self.task,
            &self.frequency,
            &self.emoji,
            &self.last_checkin,
            &self.total_checkins
        ]
    }

    fn new(streak: Streak) -> Self {
        Self {
            task: streak.task.clone(),
            frequency: streak.frequency.to_string(),
            emoji: streak.emoji_status(),
            last_checkin: {
                match streak.last_checkin {
                    Some(checkin) => checkin.to_string(),
                    None => "None".to_string()
                }
            },
            total_checkins: streak.total_checkins.to_string()
        }
    }

    fn task(&self) -> &str {
        &self.task
    }

    fn frequency(&self) -> &str {
        &self.frequency
    }

    fn emoji(&self) -> &str {
        &self.emoji
    }

    fn last_checkin(&self) -> &str {
        &self.last_checkin
    }

    fn total_checkins(&self) -> &str {
        &self.total_checkins
    }
}

fn constraint_len_calculator(items: &[Data]) -> (usize, usize, usize, usize, usize) {
    // Streak, Frequcency, Emoji, Last Checkin, Total Checkins
    let streak_len = items
        .iter()
        .map(Data::task)
        .flat_map(str::lines)
        .map(UnicodeWidthStr::width)
        .max()
        .unwrap_or(0);

    let frequency_len = items
        .iter()
        .map(Data::frequency)
        .map(UnicodeWidthStr::width)
        .max()
        .unwrap_or(0);

    let emoji_len = items
        .iter()
        .map(Data::emoji)
        .map(UnicodeWidthStr::width)
        .max()
        .unwrap_or(0);

    let last_checkin_len = items
        .iter()
        .map(Data::last_checkin)
        .map(UnicodeWidthStr::width)
        .max()
        .unwrap_or(0);

    let total_checkins_len = items
        .iter()
        .map(Data::total_checkins)
        .map(UnicodeWidthStr::width)
        .max()
        .unwrap_or(0);

    #[allow(clippy::cast_possible_truncation)]
    (streak_len as usize - 40, frequency_len as usize, emoji_len as usize, last_checkin_len as usize, total_checkins_len as usize)
}

pub fn main() -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

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

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    loop {
        if let Ok(res) = app.popup_rx.try_recv() {
            if res.0 == app.remove_popup.id && res.1 == Some(true) {
                app.remove();
            }
        }

        terminal.draw(|f| ui(f, &mut app))?;

        if let Event::Key(key) = event::read()? {
            if app.remove_popup.is_opened() && app.remove_popup.handle(key) ||
            app.create_popup.is_opened() && app.create_popup.handle(key) {
                continue;
            }

            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Char('j') | KeyCode::Down => app.next(),
                    KeyCode::Char('k') | KeyCode::Up => app.previous(),
                    KeyCode::Char('c') => app.check_in(),
                    KeyCode::Esc => {
                        app.remove_popup = app.remove_popup.close();
                        app.create_popup = app.create_popup.close();
                    },
                    KeyCode::Char('a') => {
                        app.create_popup = app
                            .create_popup
                            .modal(false)
                            .with_title(Span::styled("Add Streak", Style::default().fg(Color::Green)))
                            .with_text(Text::from(vec![
                                Line::from("Enter the task name:"),
                                Line::from("Enter the frequency:"),
                            ]))
                            .with_yes_button(ButtonLabel::new("[S]ave", 's'))
                            .with_no_button(ButtonLabel::new("[C]ancel", 'c'))
                            .with_yes_button_selected(false)
                            .with_listener(Some(app.popup_tx.clone()));
                        app.create_popup = app.create_popup.open();
                    },
                    KeyCode::Char('r') => {
                        app.remove_popup = app
                            .remove_popup
                            .modal(false)
                            .with_title(Span::styled("Remove Streak", Style::default().fg(Color::Red)))
                            .with_text(Text::from(vec![
                                Line::from("Are you sure you want to remove this streak?"),
                                Line::from(Span::styled(
                                    "This action cannot be undone.",
                                    Style::default().fg(Color::Red),
                                )),
                            ]))
                            .with_yes_button(ButtonLabel::new("[Y]es", 'y'))
                            .with_no_button(ButtonLabel::new("[N]o", 'n'))
                            .with_yes_button_selected(false)
                            .with_listener(Some(app.popup_tx.clone()));
                        app.remove_popup = app.remove_popup.open();
                    },
                    _ => {}
                }
            }
        }
    }
}

fn ui(f: &mut Frame, app: &mut App) {
    let rects = Layout::vertical([Constraint::Min(5), Constraint::Length(3)]).split(f.size());

    render_table(f, app, rects[0]);
    render_scrollbar(f, app, rects[0]);
    render_footer(f, app, rects[1]);

    let popup_rect = centered_rect(60, 40, f.size());

    if app.remove_popup.is_opened() {
        let popup = ConfirmDialog::default()
            .borders(Borders::ALL)
            .bg(Color::Black)
            .border_type(BorderType::Rounded)
            .button_style(Style::default())
            .selected_button_style(Style::default().yellow().underlined().bold());
        f.render_stateful_widget(popup, popup_rect, &mut app.remove_popup)
    }

    if app.create_popup.is_opened() {
        let popup = ConfirmDialog::default()
            .borders(Borders::ALL)
            .bg(Color::Black)
            .border_type(BorderType::Rounded)
            .button_style(Style::default())
            .selected_button_style(Style::default().yellow().underlined().bold());
        f.render_stateful_widget(popup, popup_rect, &mut app.create_popup)
    }
}

fn render_table(f: &mut Frame, app: &mut App, area: Rect) {
    let header_style = Style::default()
        .fg(app.colors.header_fg)
        .bg(app.colors.header_bg);
    let selected_style = Style::default()
        .add_modifier(Modifier::REVERSED)
        .fg(app.colors.selected_style_fg);

    let header = ["Task", "Frequency", "Status", "Last Check In", "Total"]
        .into_iter()
        .map(Cell::from)
        .collect::<Row>()
        .style(header_style)
        .height(1);

    let rows = app.items.iter().enumerate().map(|(i, data)| {
        let color = match i % 2 {
            0 => app.colors.normal_row_color,
            _ => app.colors.alt_row_color,
        };
        let mut item = data.ref_array();
        let text = item[0].clone();

        let mut wrapped_text = String::new();
        let wrapped_lines = textwrap::wrap(text.as_str(), app.longest_item_lens[0]);
        let num_lines: u16 = wrapped_lines.len().try_into().unwrap();
        for line in wrapped_lines {
            wrapped_text.push_str(&format!("{}\n", line));
        }

        item[0] = &wrapped_text;
        item.into_iter()
            .map(|content| Cell::from(Text::from(content.to_string())))
            .collect::<Row>()
            .style(Style::new().fg(app.colors.row_fg).bg(color))
            .height(num_lines)
    });

    let table = Table::new(
        rows,
        [
            Constraint::Length(app.longest_item_lens[0] as u16), // task
            Constraint::Min(app.longest_item_lens[1] as u16 + 2), // frequency
            Constraint::Min(app.longest_item_lens[2] as u16), // emoji
            Constraint::Length(app.longest_item_lens[3] as u16 + 2), // last checkin
            Constraint::Min(app.longest_item_lens[4] as u16), // total checkins
        ]
    )
    .header(header)
    .highlight_style(selected_style)
    .highlight_symbol(Text::from("> "))
    .bg(app.colors.buffer_bg)
    .highlight_spacing(HighlightSpacing::Always);
    f.render_stateful_widget(table, area, &mut app.state);
}

fn render_scrollbar(f: &mut Frame, app: &mut App, area: Rect) {
    f.render_stateful_widget(
        Scrollbar::default()
            .orientation(ScrollbarOrientation::VerticalRight)
            .begin_symbol(None)
            .end_symbol(None),
        area.inner(Margin {
            vertical: 1,
            horizontal: 1,
        }),
        &mut app.scroll_state,
    );
}

fn render_footer(f: &mut Frame, app: &App, area: Rect) {
    let info_footer = Paragraph::new(Line::from(INFO_TEXT))
        .style(Style::new().fg(app.colors.row_fg).bg(app.colors.buffer_bg))
        .centered()
        .block(
            Block::bordered()
                .border_type(BorderType::Double)
                .border_style(Style::new().fg(app.colors.footer_border_color)),
        );
    f.render_widget(info_footer, area);
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
