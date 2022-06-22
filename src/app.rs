use std::{cell::RefCell, fs::Metadata, path::PathBuf, rc::Rc};

use crossterm::event::KeyCode;
use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};
use tokio::{
    fs,
    sync::mpsc::{channel, Receiver, Sender},
};
use tui::style::Color;

use crate::task::Task;

type RcFile = Rc<RefCell<File>>;

pub struct App {
    unfiltered_files: Vec<RcFile>,
    pub files: Vec<RcFile>,
    pub current_dir: PathBuf,
    pub mode: Mode,
    pub search_term: String,
    pub split: bool,
    pub should_quit: bool,
    pub color: Color,
    rx: Receiver<StateChange>,
    sx: Sender<StateChange>,
}

impl App {
    pub fn new(path: PathBuf) -> Self {
        let (sx, rx) = channel(100);

        Self {
            unfiltered_files: vec![],
            files: vec![],
            current_dir: path,
            mode: Mode::Normal,
            search_term: String::new(),
            split: false,
            should_quit: false,
            color: Color::Green,
            rx,
            sx,
        }
    }

    pub fn tick(&mut self) {
        while let Ok(change) = self.rx.try_recv() {
            match change {
                StateChange::NewFiles(mut files) => {
                    files.sort_unstable_by_key(|f| !f.metadata.is_dir());
                    // into() instead
                    self.unfiltered_files = to_rc_file(files);
                    self.reset_filter();
                }
                StateChange::Refresh => self.get_files(self.current_dir.clone()),
            };
        }
    }

    pub fn parse_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Enter => match self.mode {
                Mode::Normal => self.open(),
                Mode::Search => {
                    self.mode = Mode::Normal;
                    self.search_term.clear();
                }
            },
            KeyCode::Esc => {
                self.mode = Mode::Normal;
                self.search_term.clear();
                self.reset_filter();
            }
            _ => match self.mode {
                Mode::Normal => match key.try_into() {
                    Ok(action) => self.take_action(action),
                    Err(()) => (),
                },
                Mode::Search => self.add_to_search(key),
            },
        }
    }

    pub fn add_to_search(&mut self, key: KeyCode) {
        match key {
            KeyCode::Backspace => {
                self.search_term.pop();
            }
            KeyCode::Char(c) => {
                self.search_term.push(c);
            }
            _ => (),
        }

        let matcher = SkimMatcherV2::default();

        let hover = self.take_hover();

        // TODO sort
        if !self.search_term.is_empty() {
            self.files = self
                .unfiltered_files
                .iter()
                .filter_map(|f| {
                    matcher
                        .fuzzy_match(&f.borrow().name, &self.search_term)
                        .filter(|score| *score > 0)
                        .map(|_| Rc::clone(f))
                })
                .collect();
        } else {
            self.reset_filter();
        }

        if let Some(file) = self.files.get(0) {
            file.borrow_mut().hovered = Some(hover.1);
        } else {
            self.unfiltered_files[0].borrow_mut().hovered = Some(hover.1);
        }
    }

    fn take_action(&mut self, action: Action) {
        match action {
            Action::Quit => self.should_quit = true,
            Action::Up => {
                let (prev_idx, hover) = self.take_hover();

                self.files[prev_idx.saturating_sub(1)].borrow_mut().hovered = Some(hover);
            }
            Action::Down => {
                let (prev_idx, hover) = self.take_hover();

                self.files[prev_idx.saturating_add(1).min(self.files.len() - 1)]
                    .borrow_mut()
                    .hovered = Some(hover);
            }
            Action::SearchMode => self.mode = Mode::Search,
            // consider ptr_eq instead of contains()
            Action::ToggleCurrent => {
                if let Some(v) = self.find_hover() {
                    let mut file = v.1.borrow_mut();
                    file.selected = !file.selected;
                }
            }
            // match self.selected.contains(self.find_hover()) {
            // true => {
            //     self.selected.drain_filter(|s| s == self.find_hover());
            // }
            // false => self.selected.push(Rc::clone(self.find_hover())),
            // },
            Action::Delete => self.delete_current(),
        }
    }

    // fn reset(&mut self) {
    //     self.search_term.clear();
    //     self.selected.clear();
    //     let hover = self.take_hover();
    //     self.unfiltered_files[0].borrow_mut().hovered = Some(hover.1);
    // }

    pub fn init(&mut self) {
        self.get_files(self.current_dir.clone());
    }

    fn get_files(&mut self, path: PathBuf) {
        Task::run(Task::GetFiles(path), self.sx.clone());
    }

    fn open(&mut self) {
        let file = self.find_hover().unwrap().1.borrow();

        let path = file.path.clone();
        if file.metadata.is_dir() {
            drop(file);
            self.get_files(path)
        } else {
            open::that(&path).unwrap();
        }
    }

    fn delete_current(&mut self) {
        let tx = self.sx.clone();
        match self.unfiltered_files.iter().any(|f| f.borrow().selected) {
            true => {
                let files = self
                    .unfiltered_files
                    .drain_filter(|f| f.borrow().selected)
                    .map(|f| (f.borrow().metadata.is_dir(), f.borrow().path.clone()))
                    .collect();

                tokio::spawn(delete_multiple(files, tx));
            }
            false => {
                let file = self
                    .unfiltered_files
                    .drain_filter(|f| f.borrow().hovered.is_some())
                    .map(|f| (f.borrow().metadata.is_dir(), f.borrow().path.clone()))
                    .next()
                    .unwrap();

                tokio::spawn(delete(file, tx));
            }
        }

        self.reset_filter();
    }

    fn reset_filter(&mut self) {
        self.files.clear();
        for file in &self.unfiltered_files {
            self.files.push(Rc::clone(file));
        }
    }

    pub fn find_hover(&self) -> Option<(usize, &RcFile)> {
        self.files
            .iter()
            .enumerate()
            .find(|(_, f)| f.borrow().hovered.is_some())
    }

    fn take_hover(&self) -> (usize, Hover) {
        let (idx, file) = self
            .files
            .iter()
            .enumerate()
            .find(|(_, f)| f.borrow().hovered.is_some())
            .unwrap();

        (idx, file.borrow_mut().hovered.take().unwrap())
    }
}

async fn delete(file: (bool, PathBuf), sx: Sender<StateChange>) {
    if file.0 {
        // TODO dont borrow
        fs::remove_dir_all(file.1).await.unwrap();
    } else {
        fs::remove_file(file.1).await.unwrap();
    }

    sx.send(StateChange::Refresh).await.unwrap();
}

async fn delete_multiple(files: Vec<(bool, PathBuf)>, sx: Sender<StateChange>) {
    for file in files {
        if file.0 {
            // TODO dont clone/borrow
            fs::remove_dir_all(file.1).await.unwrap();
        } else {
            fs::remove_file(file.1).await.unwrap();
        }
    }

    sx.send(StateChange::Refresh).await.unwrap();
}

impl TryFrom<KeyCode> for Action {
    type Error = ();

    fn try_from(value: KeyCode) -> Result<Self, Self::Error> {
        match value {
            KeyCode::Char('d') => Ok(Action::Delete),
            KeyCode::Char('t') => Ok(Action::ToggleCurrent),
            KeyCode::Char('s') | KeyCode::Char('/') => Ok(Action::SearchMode),
            KeyCode::Char('i') => Ok(Action::Up),
            KeyCode::Char('e') => Ok(Action::Down),
            KeyCode::Char('q') => Ok(Action::Quit),
            _ => Err(()),
        }
    }
}

fn to_rc_file(files: Vec<File>) -> Vec<RcFile> {
    files
        .into_iter()
        .map(RefCell::new)
        .map(Rc::new)
        .collect::<Vec<_>>()
}

#[derive(Debug)]
pub struct File {
    pub metadata: Metadata,
    pub name: String,
    pub path: PathBuf,
    pub parent: PathBuf,
    pub depth: usize,
    pub hovered: Option<Hover>,
    pub selected: bool,
}

impl PartialEq for File {
    fn eq(&self, other: &Self) -> bool {
        self.path.eq(&other.path)
    }
}

#[derive(Debug)]
pub struct Hover;

#[derive(PartialEq, Eq)]
pub enum Mode {
    Normal,
    Search,
}

#[derive(Debug)]
pub enum StateChange {
    NewFiles(Vec<File>),
    Refresh,
}

enum Action {
    Delete,

    ToggleCurrent,
    SearchMode,

    Up,
    Down,

    Quit,
}
