use std::path::PathBuf;

use tokio::sync::mpsc::Sender;
use walkdir::WalkDir;

use crate::app::{File, StateChange};

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

                    for file in walk {
                        let file = file.unwrap();
                        let metadata = file.metadata().unwrap();
                        files.push(File::new(
                            file.file_name().to_os_string().into_string().unwrap(),
                            file.depth(),
                            file.into_path(),
                            metadata.into(),
                        ));
                    }

                    tx.send(StateChange::NewFiles(files)).await.unwrap();
                }
            }
        };

        tokio::spawn(task);
    }
}
