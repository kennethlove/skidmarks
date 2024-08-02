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
    style::{self, Color, Modifier, Style, Stylize},
    terminal::{Frame, Terminal},
    text::{Line, Text},
    widgets::{
        Block, BorderType, Cell, HighlightSpacing, Paragraph, Row, Scrollbar, ScrollbarOrientation,
        ScrollbarState, Table, TableState,
    },
};
use unicode_width::UnicodeWidthStr;

const PALETTES: [tailwind::Palette; 4] = [
    tailwind::BLUE,
    tailwind::EMERALD,
    tailwind::INDIGO,
    tailwind::RED,
];

const INFO_TEXT: &str = "[Q]uit | (↑) move up | (↓) move down | [C]heck in | [A]dd | [R]emove";
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
    show_remove_popup: bool,
    show_add_popup: bool,
}

impl App {
    fn new() -> Self {
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
            colors: TableColors::new(&PALETTES[0]),
            db,
            show_remove_popup: false,
            show_add_popup: false,
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

    pub fn remove(&mut self ) {
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
    (streak_len as usize, frequency_len as usize, emoji_len as usize, last_checkin_len as usize, total_checkins_len as usize)
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
        terminal.draw(|f| ui(f, &mut app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                    KeyCode::Char('j') | KeyCode::Down => app.next(),
                    KeyCode::Char('k') | KeyCode::Up => app.previous(),
                    KeyCode::Char('c') => app.check_in(),
                    KeyCode::Char('a') => {
                        // Add a new streak
                    },
                    KeyCode::Char('r') => app.remove(),
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
        let item = data.ref_array();
        item.into_iter()
            .map(|content| Cell::from(Text::from(format!("{content}\n"))))
            .collect::<Row>()
            .style(Style::new().fg(app.colors.row_fg).bg(color))
            .height(1)
    });

    let table = Table::new(
        rows,
        [
            Constraint::Length(app.longest_item_lens[0] as u16 + 1),
            Constraint::Min(app.longest_item_lens[1] as u16 + 1),
            Constraint::Min(app.longest_item_lens[2] as u16),
            Constraint::Min(app.longest_item_lens[3] as u16),
            Constraint::Min(app.longest_item_lens[4] as u16),
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
