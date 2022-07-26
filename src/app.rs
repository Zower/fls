use colorsys::Rgb;
use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};
use iced::{
    executor,
    keyboard::{KeyCode, Modifiers},
    pure::{Application, Element},
    Command,
};
use iced_native::{event::Status, keyboard::Event, subscription::events_with};
use std::{fs::Metadata, ops::Index, path::PathBuf};

use tokio::fs::remove_file;

use crate::{
    mode::Mode,
    tasks::get_files,
    theme::{RatioExt, Theme},
    ui::{self},
};

#[derive(Debug, Clone)]
pub enum Message {
    FilesLoaded(Vec<File>),
    KeyEvent(Event),
    FileDeleteResult(Result<(), FileDeleteError>),
    ColorInput(SettingsInputKind, String),
    SubmitColor(SettingsInputKind),
}

#[derive(Debug, Clone)]
pub struct FileDeleteError(pub PathBuf);

#[derive(Debug)]
pub struct DisplayedFile {
    pub curr_score: i64,
    pub data: File,
    // pub hovered: bool,
    pub selected: bool,
}

#[derive(Debug)]
pub struct Files(Vec<DisplayedFile>);
impl Files {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn remove(&mut self, index: usize) -> DisplayedFile {
        self.0.remove(index)
    }

    pub fn drain(&mut self, p: impl Fn(&DisplayedFile) -> bool) -> Vec<DisplayedFile> {
        let mut vec = Vec::new();
        let mut i = 0;
        while i < self.0.len() {
            let item = &mut self.0[i];
            if item.curr_score > 0 && p(item) {
                vec.push(self.0.remove(i));
            } else {
                i += 1;
            }
        }

        vec
    }

    pub fn files(&self) -> impl Iterator<Item = &DisplayedFile> {
        self.0.iter().filter(|f| f.curr_score > 0)
    }

    pub fn files_mut(&mut self) -> impl Iterator<Item = &mut DisplayedFile> {
        self.0.iter_mut().filter(|f| f.curr_score > 0)
    }

    pub fn set(&mut self, files: Vec<DisplayedFile>) {
        self.0 = files;
    }

    pub(super) fn new_scores(&mut self, score_fn: impl Fn(&File) -> i64) {
        self.0
            .iter_mut()
            .for_each(|f| f.curr_score = score_fn(&f.data));
    }
}

impl Index<usize> for Files {
    type Output = DisplayedFile;

    fn index(&self, index: usize) -> &Self::Output {
        &self
            .files()
            .skip(index)
            .next()
            .expect("index out of bounds")
    }
}

pub struct Fls {
    pub current_dir: PathBuf,
    pub mode: Mode,
    pub search_term: String,
    pub hovered: usize,
    pub should_exit: bool,
    pub curr_view: View,
    pub theme: Theme,
    cache: Files,
}

impl Application for Fls {
    type Executor = executor::Default;
    type Message = Message;
    // TODO: Create a config and add it here
    type Flags = PathBuf;

    type Theme = Theme;

    fn update(&mut self, message: Self::Message) -> iced::Command<Self::Message> {
        if 1 == 2 % 3 {
            self.curr_view = View::Settings(SettingsView::default());
        }

        let mut command = Command::none();

        match message {
            Message::FilesLoaded(f) => self.cache.set(f.into_iter().map(Into::into).collect()),
            Message::KeyEvent(e) => {
                let action = match self.curr_view {
                    View::MainView => self.mode.parse_event(e),
                    View::Settings { .. } => View::parse_settings(e),
                };

                command = self.take_action(action);
            }
            Message::FileDeleteResult(r) => r.unwrap(),
            Message::SubmitColor(id) => match &mut self.curr_view {
                View::Settings(s) => match id {
                    SettingsInputKind::PrimaryColor => {
                        self.theme.primary = Rgb::from_hex_str(&s.primary_input)
                            .unwrap()
                            .as_ratio()
                            .to_color()
                    }
                    SettingsInputKind::SecondaryColor => {
                        self.theme.secondary = Rgb::from_hex_str(&s.secondary_input)
                            .unwrap()
                            .as_ratio()
                            .to_color()
                    }
                },
                _ => unreachable!(),
            },
            Message::ColorInput(id, string) => match &mut self.curr_view {
                View::Settings(s) => match id {
                    SettingsInputKind::PrimaryColor => s.primary_input = string,
                    SettingsInputKind::SecondaryColor => s.secondary_input = string,
                },
                _ => unreachable!(),
            },
        }
        command
    }

    fn subscription(&self) -> iced::Subscription<Self::Message> {
        events_with(|e, s| match e {
            iced_native::Event::Keyboard(e) if s == Status::Ignored => Some(Message::KeyEvent(e)),
            // TODO: Could be useful for drag and drop files? idk
            // Event::Window()
            _ => None,
        })
    }

    fn view(&self) -> Element<'_, Message, iced::Renderer<Theme>> {
        ui::draw(&self)
    }

    fn new(flags: PathBuf) -> (Self, Command<Message>) {
        let app = Fls {
            cache: Files::new(),
            current_dir: flags,
            mode: Mode::Normal,
            search_term: String::new(),
            hovered: 0,
            // pane: state,
            // mode: Mode::Normal,
            should_exit: false,
            curr_view: View::MainView,
            theme: Theme::default(),
        };

        let dir = app.current_dir.clone();
        (
            app,
            Command::perform(get_files(dir), |f| Message::FilesLoaded(f)),
        )
    }

    fn title(&self) -> String {
        "FLS".to_string()
    }

    fn should_exit(&self) -> bool {
        self.should_exit
    }

    fn theme(&self) -> Self::Theme {
        self.theme
    }
}

impl Fls {
    pub fn files(&self) -> impl Iterator<Item = &DisplayedFile> {
        self.cache.files()
    }

    pub fn files_mut(&mut self) -> impl Iterator<Item = &mut DisplayedFile> {
        self.cache.files_mut()
    }

    fn take_action(&mut self, action: Action) -> Command<Message> {
        let mut command = Command::none();

        match action {
            Action::Quit => self.should_exit = true,
            Action::Up => {
                self.hovered = self.hovered.saturating_sub(1);
            }
            Action::Down => {
                self.hovered = self
                    .hovered
                    .saturating_add(1)
                    .min(self.files().count().saturating_sub(1));
            }
            Action::NewMode(m) => {
                match m {
                    Mode::Normal => self.search_term.clear(),
                    _ => (),
                }

                self.mode = m;
                self.refresh_filter();
            }
            Action::ToggleCurrent => {
                let hovered = self.hovered;
                let _ = self.files_mut().skip(hovered).next().map(|f| {
                    f.selected = !f.selected;
                });
            }
            Action::Delete => {
                // TODO: Double clone is unideal but closures make rust stuped
                let create_command = |path: PathBuf| {
                    Command::perform(remove_file(path.clone()), move |r| {
                        let clone = path.clone();
                        Message::FileDeleteResult(r.map_err(|_| FileDeleteError(clone)))
                    })
                };

                //todo prob deletes once that shouldnt be
                if self.files().any(|f| f.selected) {
                    let files = self.cache.drain(|f| f.selected);
                    command = Command::batch(files.into_iter().map(|f| create_command(f.data.path)))
                } else {
                    let file = self.cache.remove(self.hovered);

                    command = create_command(file.data.path)
                }

                let count = self.files().count() - 1;

                self.hovered = if self.hovered < count {
                    self.hovered
                } else {
                    count
                }
            }
            Action::Open => {
                let file = &self.cache[self.hovered];
                let path = &file.data.path;
                if file.data.metadata.is_dir() {
                    let (new, c) = Self::new(path.clone());

                    *self = new;
                    command = c;
                } else {
                    open::that(&path).unwrap();
                }
            }
            Action::UpDir => {
                let (new, c) = Self::new(self.current_dir.parent().unwrap().into());

                *self = new;
                command = c;
            }
            Action::AddToSearch(c) => {
                self.search_term.push(c);
                self.refresh_filter()
            }
            Action::PopFromSearch => {
                let x = self.search_term.pop();
                println!("search_term: {:?}, {x:?}", self.search_term);
                self.refresh_filter()
            }
            Action::FreezeSearch => {
                self.mode = Mode::Normal;
                self.search_term.clear();
            }
            Action::None => (),
            Action::NewView(view) => self.curr_view = view,
        }

        command
    }

    fn refresh_filter(&mut self) {
        let matcher = SkimMatcherV2::default();

        // TODO sort
        if !self.search_term.is_empty() {
            self.cache.new_scores(|f| {
                matcher
                    .fuzzy_match(&f.name, &self.search_term)
                    .unwrap_or(-1)
            })
        } else {
            self.cache.new_scores(|_| i64::MAX);
        }

        self.hovered = 0;
    }
}

#[derive(Debug, Clone)]
pub struct File {
    pub metadata: Metadata,
    pub name: String,
    pub path: PathBuf,
    pub parent: PathBuf,
    pub depth: usize,
}

impl File {
    pub fn new(
        name: String,
        depth: usize,
        path: PathBuf,
        parent: PathBuf,
        metadata: Metadata,
    ) -> Self {
        Self {
            name,
            depth,
            path,
            parent,
            metadata,
        }
    }
}

impl PartialEq for File {
    fn eq(&self, other: &Self) -> bool {
        self.path.eq(&other.path)
    }
}

impl From<File> for DisplayedFile {
    fn from(f: File) -> Self {
        Self {
            data: f,
            curr_score: i64::MAX,
            selected: false,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Action {
    Up,
    Down,
    UpDir,
    Open,

    Delete,
    ToggleCurrent,

    NewMode(Mode),
    NewView(View),
    AddToSearch(char),
    PopFromSearch,
    FreezeSearch,

    Quit,
    None,
}

#[derive(Debug, Clone)]
pub enum View {
    MainView,
    Settings(SettingsView),
}

#[derive(Debug, Copy, Clone)]
pub enum SettingsInputKind {
    PrimaryColor,
    SecondaryColor,
}

#[derive(Debug, Clone, Default)]
pub struct SettingsView {
    pub primary_input: String,
    pub secondary_input: String,
}

impl View {
    fn parse_settings(event: Event) -> Action {
        if let Event::KeyPressed {
            key_code,
            modifiers,
        } = event
        {
            match key_code {
                KeyCode::S if modifiers.contains(Modifiers::CTRL) => {
                    Action::NewView(View::MainView)
                }
                _ => Action::None,
            }
        } else {
            Action::None
        }
    }
}
