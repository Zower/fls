use std::path::PathBuf;

use tokio::sync::mpsc::Sender;
use walkdir::WalkDir;

use crate::app::{File, Hover, StateChange};

pub enum Task {
    GetFiles(PathBuf),
}

impl Task {
    pub fn run(self, tx: Sender<StateChange>) {
        let task = match self {
            Task::GetFiles(path) => {
                async move {
                    let walk = WalkDir::new(path).min_depth(1).max_depth(1);
                    // let mut dir = fs::read_dir("aa").await.unwrap();

                    let mut files = vec![];

                    let mut first = true;
                    for file in walk {
                        let file = file.unwrap();
                        let metadata = file.metadata().unwrap();
                        files.push(File {
                            name: file.file_name().to_os_string().into_string().unwrap(),
                            depth: file.depth(),
                            path: file.into_path(),
                            parent: "".into(),
                            metadata: metadata.into(),
                            hovered: if first {
                                first = false;
                                Some(Hover)
                            } else {
                                None
                            },
                            selected: false,
                        });
                    }
                    // while let Ok(Some(file)) = dir.next_entry().await {
                    // }

                    tx.send(StateChange::NewFiles(files)).await.unwrap();
                }
            }
        };

        tokio::spawn(task);
    }
}
