use iced::alignment::{self, Alignment};
use iced::font::{self, Font};
use iced::keyboard;
use iced::theme::{self, Theme};
use iced::widget::{
    self, button, checkbox, column, container, keyed_column, row, scrollable,
    text, text_input, Text
};
use iced::window;
use iced::{Application, Color, Command, Element, Length, Settings, Size, Subscription};

use once_cell::sync::Lazy;
use ratatui::layout::Constraint::Length;
use ron::Error::Message;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

static INPUT_ID: Lazy<text_input::Id> = Lazy::new(text_input::Id::unique);
const ICONS: Font = Font::with_name("Iced-Todos-Icons");

pub fn main() -> iced::Result {
    #[cfg(not(target_arch = "wasm32"))]
    tracing_subscriber::fmt::init();

    Todos::run(Settings {
        window: window::Settings {
            size: Size::new(500.0, 800.0),
            ..window::Settings::default()
        },
        ..Settings::default()
    })
}

#[derive(Debug)]
enum Todos {
    Loading,
    Loaded(State),
}

#[derive(Debug, Default)]
struct State {
    input_value: String,
    filter: Filter,
    tasks: Vec<Task>,
    dirty: bool,
    saving: bool,
}

#[derive(Clone, Debug)]
enum Message {
    Loaded(Result<SavedState, LoadError>),
    FontLoaded(Result<(), font::Error>),
    Saved(Result<(), SaveError>),
    InputChanged(String),
    CreateTask,
    FilterChanged(Filter),
    TaskMessage(usize, TaskMessage),
    TabPressed { shift: bool },
    ToggleFullscreen(window::Mode),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct Task {
    #[serde(default="Uuid::new_v4")]
    id: Uuid,
    description: String,
    completed: bool,

    #[serde(skip)]
    state: TaskState,
}

#[derive(Clone, Debug)]
pub enum TaskState {
    Idle,
    Editing,
}

impl Default for TaskState {
    fn default() -> Self {
        Self::Idle
    }
}

#[derive(Clone, Debug)]
pub enum TaskMessage {
    Completed(bool),
    Edit,
    DescriptionEdited(String),
    FinishEdition,
    Delete,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub enum Filter {
    #[default]
    All,
    Active,
    Completed
}

impl Filter {
    fn matches(self, task: &Task) -> bool {
        match self {
            Filter::All => true,
            Filter::Active => !task.completed,
            Filter::Completed => task.completed,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct SavedState {
    input_value: String,
    filter: Filter,
    tasks: Vec<Task>,
}

#[derive(Clone, Debug)]
enum LoadError {
    File,
    Format,
}

#[derive(Clone, Debug)]
enum SaveError {
    File,
    Write,
    Format,
}

#[cfg(not(target_arch = "wasm32"))]
impl SavedState {
    fn path() -> std::path::PathBuf {
        let mut path = if let Some(project_dirs) =
            directories_next::ProjectDirs::from("rs", "Iced", "Todos")
        {
            project_dirs.data_dir().into()
        } else {
            std::env::current_dir().unwrap_or_default()
        };

        path.push("todos.json");

        path
    }

    async fn load() -> Result<SavedState, LoadError> {
        use async_std::prelude::*;

        let mut contents = String::new();

        let mut file = async_std::fs::File::open(Self::path())
            .await
            .map_err(|_| LoadError::File)?;

        file.read_to_string(&mut contents)
            .await
            .map_err(|_| LoadError::File)?;

        serde_json::from_str(&contents).map_err(|_| LoadError::Format)
    }

    async fn save(self) -> Result<(), SaveError> {
        use async_std::prelude::*;

        let json = serde_json::to_string_pretty(&self)
            .map_err(|_| SaveError::Format)?;

        let path = Self::path();

        if let Some(dir) = path.parent() {
            async_std::fs::crate_dir_all(dir)
                .await
                .map_err(|_| SaveError::File)?;
        }

        {
            let mut file = async_std::fs::File::create(path)
                .await
                .map_err(|_| SaveError::File)?;

            file.write_all(json.as_bytes())
                .await
                .map_err(|_| SaveError::Write)?;
        }

        async_std::task::sleep(std::time::Duration::from_secs(2)).await;

        Ok(())
    }
}

fn icon(unicode: char) -> Text<'static> {
    text(unicode.to_string())
        .font(ICONS)
        .width(20)
        .horizontal_alignment(alignment::Horizontal::Center)
}

fn edit_icon() -> Text<'static> {
    icon('\u{F303}')
}

fn delete_icon() -> Text<'static> {
    icon('\u{F1F8}')
}

fn loading_message<'a>() -> Element<'a, Message> {
    container(
        text("Loading...")
            .horizontal_alignment(alignment::Horizontal::Center)
            .size(50),
    )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_y()
        .into()
}

fn empty_message(message: &str) -> Element<'_, Message> {
    container(
        text(message)
            .width(Length::Fill)
            .size(25)
            .horizontal_alignment(alignment::Horizontal::Center)
            .style(Color::from([0.7, 0.7, 0.7])),
    )
        .height(200)
        .center_y()
        .into()
}

fn view_controls(tasks: &[Task], current_filter: Filter) -> Element<Message> {
    let tasks_left = tasks.iter().filter(|task| !task.completed).count();

    let filter_button = |label, filter, current_filter| {
        let label = text(label);

        let button = button(label).style(if filter == current_filter {
            theme::Button::Primary
        } else {
            theme::Button::Text
        });

        button.on_press(Message::FilterChanged(filter)).padding(8)
    };

    row![
        text(format!(
            "{tasks_left} {} left",
            if tasks_left == 1 { "task" } else { "tasks" }
        ))
        .width(Length::Fill),
        row![
            filter_button("All", Filter::All, current_filter),
            filter_button("Active", Filter::Active, current_filter),
            filter_button("Completed", Filter::Completed, current_filter),
        ]
        .width(Length::Shrink)
        .spacing(10)
    ]
        .spacing(20)
        .align_items(Alignment::Center)
        .into()
}

impl Application for Todos {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Todos, Command<Message>) {
        (
            Todos::Loading,
            Command::batch(vec![
                font::load(include_bytes!("../fonts/icons.ttf").as_slice())
                    .map(Message::FontLoaded),
                Command::perform(SavedState::load(), Message::Loaded),
            ]),
        )
    }

    fn title(&self) -> String {
        let dirty = match self {
            Todos::Loading => false,
            Todos::Loaded(state) => state.dirty,
        };

        format!("Todos{} - Iced", if dirty { "*" } else { "" })
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match self {
            Todos::Loading => {
                match message {
                    Message::Loaded(Ok(state)) => {
                        *self = Todos::Loaded(State {
                            input_value: state.input_value,
                            filter: state.filter,
                            tasks: state.tasks,
                            ..State::default()
                        });
                    }
                    Message::Loaded(Err(_)) => {
                        *self = Todos::Loaded(State::default());
                    }
                    _ => {}
                }

                text_input::focus(INPUT_ID.clone())
            }
            Todos::Loaded(state) => {
                let mut saved = false;

                let command = match message {
                    Message::InputChanged(value) => {
                        state.input_value = value;

                        Command::none()
                    }
                    Message::CreateTask => {
                        if !state.input_value.is_empty() {
                            state
                                .tasks
                                .push(Task::new(state.input_value.clone()));
                            state.input_value.clear();
                        }
                        Command::none()
                    }
                    Message::FilterChanged(filter) => {
                        state.filter = filter;

                        Command::none()
                    }
                    Message::TaskMessage(i, TaskMessage::Delete) => {
                        state.tasks.remove(i);

                        Command::none()
                    }
                    Message::TaskMessage(i, task_message) => {
                        if let Some(task) = state.tasks.get_mut(i) {
                            let should_focus = matches!(task_message, TaskMessage::Edit);

                            task.update(task_message);

                            if should_focus {
                                let id = Task::text_input_id(i);
                                Command::batch(vec![
                                    text_input::focus(id.clone()),
                                    text_input::select_all(id),
                                ])
                            } else {
                                Command::none()
                            }
                        } else {
                            Command::none()
                        }
                    }
                    Message::Saved(_result) => {
                        state.saving = false;
                        saved = true;

                        Command::none()
                    }
                    Message::TabPressed { shift } => {
                        if shift {
                            widget::focus_previous()
                        } else {
                            widget::focus_next()
                        }
                    }
                    Message::ToggleFullscreen(mode) => {
                        window::change_mode(window::Id::MAIN, mode)
                    }
                    _ => Command::none(),
                };

                if !saved {
                    state.dirty = true;
                }

                let save = if state.dirty && !state.saving {
                    state.dirty = false;
                    state.saving = true;

                    Command::perform(
                        SavedState {
                            input_value: state.input_value.clone(),
                            filter: state.filter,
                            tasks: state.tasks.clone(),
                        }
                            .save(),
                        Message::Saved,
                    )
                } else {
                    Command::none()
                };

                Command::batch(vec![command, save])
            }
        }
    }

    fn view(&self) -> Element<Message> {
        match self {
            Todos::Loading => loading_message(),
            Todos::Loaded(State {
                              input_value,
                              filter,
                              tasks,
                              ..
                          }) => {
                let title = text("todos")
                    .width(Length::Fill)
                    .size(100)
                    .style(Color::from([0.5, 0.5, 0.5]))
                    .horizontal_alignment(alignment::Horizontal::Center);

                let input = text_input("What needs to be done?", input_value)
                    .id(INPUT_ID.clone())
                    .on_input(Message::InputChanged)
                    .on_submit(Message::CreateTask)
                    .padding(15)
                    .size(30);

                let controls = view_controls(tasks, *filter);
                let filtered_tasks =
                    tasks.iter().filter(|task| filter.matches(task));

                let tasks: Element<_> = if filtered_tasks.count() > 0 {
                    keyed_column(
                        tasks
                            .iter()
                            .enumerate()
                            .filter(|_, task| filter.matches(task))
                            .map(|(i, task)| {
                                (
                                    task.id,
                                    task.view(i).map(move |message| {
                                        Message::TaskMessage(i, message)
                                    }),
                                )
                            }),
                    )
                        .spacing(10)
                        .into()
                } else {
                    empty_message(match filter {
                        Filter::All => "You have not created a task yet...",
                        Filter::Active => "All your tasks are done! :D",
                        Filter::Completed => {
                            "You have not completed a task yet..."
                        }
                    })
                };

                let content = column![title, input, controls, tasks]
                    .spacing(20)
                    .max_width(800);

                scrollable(container(content).padding(40).center_x()).into()
            }
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        use keyboard::key;

        keyboard::on_key_press(|key, modifiers| {
            let keyboard::Key::Named(key) = key else {
                return None;
            };

            match (key, modifiers) {
                (key::Named::Tab, _) => Some(Message::TabPressed {
                    shift: modifiers.shift(),
                }),
                (key::Named::ArrowUp, keyboard::Modifiers::SHIFT) => {
                    Some(Message::ToggleFullscreen(window::Mode::Fullscreen))
                }
                (key::Named::ArrowDown, keyboard::Modifiers::SHIFT) => {
                    Some(Message::ToggleFullscreen(window::Mode::Windowed))
                }
                _ => None,
            }
        })
    }
}