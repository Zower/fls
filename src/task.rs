use std::path::PathBuf;

use jwalk::WalkDir;
use tokio::sync::mpsc::Sender;

use crate::app::{File, Message};

pub enum Task {
    GetFiles(PathBuf),
}

impl Task {
    pub fn run(self, tx: Sender<Message>) {
        let task = match self {
            Task::GetFiles(path) => {
                async move {
                    let walk = WalkDir::new(path)
                        .skip_hidden(false)
                        .min_depth(1)
                        .max_depth(1);
                    // let mut dir = fs::read_dir("aa").await.unwrap();

                    let mut files = vec![];

                    for file in walk {
                        let file = file.unwrap();
                        let metadata = file.metadata().unwrap();
                        files.push(File::new(
                            file.file_name().to_os_string().into_string().unwrap(),
                            file.depth(),
                            file.path(),
                            file.parent_path().into(),
                            metadata.into(),
                        ));
                    }

                    tx.send(Message::NewFiles(files)).await.unwrap();
                }
            }
        };

        tokio::spawn(task);
    }
}
