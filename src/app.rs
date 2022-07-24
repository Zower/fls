use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};
use iced::{
    executor,
    pure::{Application, Element},
    Command,
};
use iced_native::{event::Status, keyboard::Event, subscription::events_with};
use std::{fs::Metadata, ops::Index, path::PathBuf};

use crate::{
    mode::{Mode, SearchMode},
    tasks::get_files,
    theme::Theme,
    ui,
};

#[derive(Debug, Clone)]
pub enum Message {
    FilesLoaded(Vec<File>),
    KeyEvent(Event),
    Button,
}

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
    cache: Files,
    pub current_dir: PathBuf,
    pub mode: Mode,
    pub search_term: String,
    pub hovered: usize,
    pub should_exit: bool,
    // pub pane: pane_grid::State<PaneState>, // rx: Receiver<Message>,
}

impl Application for Fls {
    type Executor = executor::Default;
    type Message = Message;
    // TODO: Create a config and add it here
    type Flags = PathBuf;

    type Theme = Theme;

    fn update(&mut self, message: Self::Message) -> iced::Command<Self::Message> {
        let mut command = Command::none();

        match message {
            Message::FilesLoaded(f) => self.cache.set(f.into_iter().map(Into::into).collect()),
            Message::KeyEvent(e) => {
                let action = self.mode.parse_event(e);

                command = self.take_action(action);
            }
            Message::Button => {
                command = self.take_action(Action::NewMode(Mode::Search(SearchMode::Regular)))
            }
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
        Theme::Default
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
        println!("take_action: {:?}", action);

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
            Action::Delete => (),
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

//     fn delete_current(&mut self) {
//         let tx = self.sx.clone();

//         let old_idx = self.find_hover().map(|(i, _)| i).unwrap_or(0);

//         match self.unfiltered_files.iter().any(|f| f.borrow().selected) {
//             true => {
//                 let files = self
//                     .unfiltered_files
//                     .drain_filter(|f| f.borrow().selected)
//                     .map(|f| (f.borrow().metadata.is_dir(), f.borrow().path.clone()))
//                     .collect();

//                 tokio::spawn(delete_multiple(files, tx));
//             }
//             false => {
//                 let file = self
//                     .unfiltered_files
//                     .drain_filter(|f| f.borrow().hovered)
//                     .map(|f| (f.borrow().metadata.is_dir(), f.borrow().path.clone()))
//                     .next()
//                     .unwrap();

//                 tokio::spawn(delete(file, tx));
//             }
//         }

//         self.files.clear();
//         for file in &self.unfiltered_files {
//             self.files.push(Rc::clone(file));
//         }

//         let idx = if self.files.get(old_idx).is_some() {
//             old_idx
//         } else if self.files.get(old_idx.saturating_sub(1)).is_some() {
//             old_idx.saturating_sub(1)
//         } else {
//             0
//         };

//         self.move_hover(idx);
//     }

// async fn delete(file: (bool, PathBuf), _sx: Sender<Message>) {
//     if file.0 {
//         fs::remove_dir_all(file.1).await.unwrap();
//     } else {
//         fs::remove_file(file.1).await.unwrap();
//     }

//     // sx.send(StateChange::Refresh).await.unwrap();
// }

// async fn delete_multiple(files: Vec<(bool, PathBuf)>, _sx: Sender<Message>) {
//     for file in files {
//         if file.0 {
//             // TODO dont clone/borrow
//             fs::remove_dir_all(file.1).await.unwrap();
//         } else {
//             fs::remove_file(file.1).await.unwrap();
//         }
//     }

//     // sx.send(StateChange::Refresh).await.unwrap();

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

#[derive(Debug, Clone, Copy)]
pub enum Action {
    Up,
    Down,
    UpDir,
    Open,

    Delete,
    ToggleCurrent,

    NewMode(Mode),
    AddToSearch(char),
    PopFromSearch,
    FreezeSearch,

    Quit,
    None,
}
