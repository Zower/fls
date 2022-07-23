use iced::{
    executor, keyboard,
    pure::{
        text,
        widget::{pane_grid, Column, Container, PaneGrid, Text},
        Application, Element,
    },
    Command, Length,
};
use iced_native::{event::Status, subscription::events_with, Event};
use ouroboros::self_referencing;
use std::{fs::Metadata, path::PathBuf};

use crate::tasks::get_files;

#[derive(Debug)]
pub enum Message {
    FilesLoaded(Vec<File>),
    // FileSearchDone(Vec<File>),
    // Test(u32),
    KeyEvent(keyboard::Event),
    Clicked(pane_grid::Pane),
    Dragged(pane_grid::DragEvent),
    Resized(pane_grid::ResizeEvent),
}

pub struct Fls {
    files: Files,
    pub current_dir: PathBuf,
    // pub mode: Mode,
    // pub search_term: String,
    // pub split: bool,
    pub should_quit: bool,
    pub pane: pane_grid::State<PaneState>, // rx: Receiver<Message>,
                                           // key_rx: Receiver<KeyEvent>,
                                           // sx: Sender<Message>,
}

// #[derive(Debug)]
#[self_referencing]
struct Files {
    files: Vec<File>,
    #[borrows(files)]
    #[covariant]
    displayed_files: Vec<&'this File>,
}

pub enum PaneState {
    SomePane,
    AnotherKindOfPane,
}

impl Application for Fls {
    type Executor = executor::Default;
    type Message = Message;
    // TODO: Create a config and add it here
    type Flags = PathBuf;

    fn new(flags: PathBuf) -> (Self, Command<Message>) {
        // let (sx, rx) = channel(100)
        let (mut state, pane) = pane_grid::State::new(PaneState::SomePane);

        state.split(
            iced::pane_grid::Axis::Vertical,
            &pane,
            PaneState::AnotherKindOfPane,
        );

        let app = Fls {
            files: Files::new(Vec::new(), |f| f.iter().collect::<Vec<_>>()),
            pane: state,
            current_dir: flags,
            // mode: Mode::Normal,
            should_quit: false,
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

    fn background_color(&self) -> iced::Color {
        iced::Color::from_rgb(0.29, 0.3, 0.41)
    }

    fn update(&mut self, message: Self::Message) -> iced::Command<Self::Message> {
        // let x = self.borrow_displayed_files();
        println!("{:?}", message);
        Command::none()
    }

    fn subscription(&self) -> iced::Subscription<Self::Message> {
        events_with(|e, s| match e {
            Event::Keyboard(e) if s == Status::Ignored => Some(Message::KeyEvent(e)),
            // TODO: Could be useful for drag and drop files? idk
            // Event::Window()
            _ => None,
        })
    }

    fn view(&self) -> Element<Message> {
        // let pane_grid = PaneGrid::new(&self.pane, |pane, state| {
        //     pane_grid::Content::new(match state {
        //         PaneState::SomePane => text("This is some pane"),
        //         PaneState::AnotherKindOfPane => text("This is another kind of pane"),
        //     })
        // })
        // .on_drag(Message::Dragged)
        // .on_click(Message::Clicked)
        // .on_resize(10, Message::Resized);

        Column::new()
            .width(Length::Fill)
            .height(Length::Fill)
            .push(text("Hello, world!"))
            .push(text("Hello, pord!"))
            // .center_x()
            // .center_y()
            .into()
        // Text::new("test").into()
    }

    // pub fn tick(&mut self) {
    // while let Ok(change) = self.rx.try_recv() {
    //     match change {
    //         Message::NewFiles(mut files) => {
    //             files.sort_unstable_by_key(|f| !f.metadata.is_dir());
    //             files[0].hovered = true;

    //             // into() instead
    //             self.unfiltered_files = to_rc_file(files);
    //             self.reset_filter();
    //         }
    //         _ => (), // StateChange::Refresh => self.get_files(self.current_dir.clone()),
    //     };
    // }

    // while let Ok(key) = self.key_rx.try_recv() {
    //     self.parse_key(key);
    // }
}

// Todo should delegate to each mode
// pub fn parse_key(&mut self, key: KeyEvent) {
//     let KeyEvent { code, modifiers: _ } = key;

//     match code {
//         KeyCode::Esc => {
//             self.mode = Mode::Normal;
//             self.search_term.clear();
//             self.reset_filter();
//         }
//         _ => match self.mode.parse_key(key) {
//             Some(action) => self.take_action(action),
//             None => (),
//         },
//     }
// }

// fn refresh_filter(&mut self) {
//     let matcher = SkimMatcherV2::default();

//     // TODO sort
//     if !self.search_term.is_empty() {
//         self.files = self
//             .unfiltered_files
//             .iter()
//             .filter_map(|f| {
//                 matcher
//                     .fuzzy_match(&f.borrow().name, &self.search_term)
//                     .filter(|score| *score > 0)
//                     .map(|_| Rc::clone(f))
//             })
//             .collect();
//     } else {
//         self.reset_filter();
//     }

//     self.move_hover(0);
// }

// fn take_action(&mut self, action: Action) {
//     match action {
//         Action::Quit => self.should_quit = true,
//         Action::Up => {
//             if let Some((prev_idx, _)) = self.find_hover() {
//                 self.move_hover(prev_idx.saturating_sub(1))
//             } else {
//                 if let Some(file) = self.files.get(0) {
//                     file.borrow_mut().hovered = true;
//                 }
//             }
//         }
//         Action::Down => {
//             if let Some((prev_idx, _)) = self.find_hover() {
//                 self.move_hover(
//                     prev_idx
//                         .saturating_add(1)
//                         .min(self.files.len().saturating_sub(1)),
//                 )
//             } else {
//                 if let Some(file) = self.files.get(0) {
//                     file.borrow_mut().hovered = true;
//                 }
//             }
//         }
//         Action::SearchMode(m) => self.mode = Mode::Search(m),
//         // consider ptr_eq instead of contains()
//         Action::ToggleCurrent => {
//             if let Some(v) = self.find_hover() {
//                 let mut file = v.1.borrow_mut();
//                 file.selected = !file.selected;
//             }
//         }
//         Action::Delete => self.delete_current(),
//         Action::Open => self.open(),
//         Action::Back => self.go_up_dir(),
//         Action::AddToSearch(c) => {
//             self.search_term.push(c);
//             self.refresh_filter()
//         }
//         Action::PopFromSearch => {
//             self.search_term.pop();
//             self.refresh_filter()
//         }
//         Action::FreezeSearch => {
//             self.mode = Mode::Normal;
//             self.search_term.clear();
//         }
//     }
// }

// fn get_files(&self, path: PathBuf) {
//     Task::run(Task::GetFiles(path), self.sx.clone());
// }
//
// fn open(&mut self) {
//     let file = self.find_hover().unwrap().1.borrow();

//     let path = file.path.clone();
//     if file.metadata.is_dir() {
//         drop(file);
//         self.current_dir = path;
//         self.get_files(self.current_dir.clone())
//     } else {
//         drop(file);
//         open::that(&path).unwrap();
//     }

//     self.search_term.clear();
//     self.reset_filter();
// }
//
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

//     fn reset_filter(&mut self) {
//         self.files.clear();
//         for file in &self.unfiltered_files {
//             self.files.push(Rc::clone(file));
//         }

//         self.move_hover(0);
//     }

//     pub fn find_hover(&self) -> Option<(usize, &RcFile)> {
//         self.files
//             .iter()
//             .enumerate()
//             .find(|(_, f)| f.borrow().is_hovered())
//     }

//     fn move_hover(&mut self, new: usize) {
//         if let Some(file) = self.find_hover() {
//             file.1.borrow_mut().hovered = false;
//         }

//         if let Some(val) = self.files.get(new) {
//             val.borrow_mut().hovered = true;
//         }
//     }

//     fn go_up_dir(&mut self) {
//         self.current_dir.pop();
//         self.get_files(self.current_dir.clone());

//         self.search_term.clear();
//         self.reset_filter();
//     }
// }

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
// }

#[derive(Debug)]
pub struct File {
    pub metadata: Metadata,
    pub name: String,
    pub path: PathBuf,
    pub parent: PathBuf,
    pub depth: usize,
    pub hovered: bool,
    pub selected: bool,
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
            hovered: false,
            selected: false,
        }
    }

    pub fn is_hovered(&self) -> bool {
        self.hovered
    }
}

impl PartialEq for File {
    fn eq(&self, other: &Self) -> bool {
        self.path.eq(&other.path)
    }
}

pub enum Action {
    Up,
    Down,
    Back,
    Open,

    Delete,
    ToggleCurrent,
    // SearchMode(SearchMode),
    AddToSearch(char),
    PopFromSearch,
    FreezeSearch,

    Quit,
}
