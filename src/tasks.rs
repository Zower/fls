use std::path::PathBuf;

// use tokio::sync::mpsc::Sender;
use tokio::fs::read_dir;

use crate::app::File;

pub async fn get_files(dir: PathBuf) -> Vec<File> {
    // let walk = WalkDir::new(path)
    //     .skip_hidden(false)
    //     .min_depth(1)
    //     .max_depth(1);

    // let mut dir = fs::read_dir("aa").await.unwrap();

    let mut files = vec![];

    let mut dir = read_dir(dir).await.unwrap();

    loop {
        match dir.next_entry().await {
            Ok(Some(f)) => files.push(File::new(
                f.file_name().to_str().unwrap().to_string(),
                0,
                f.path(),
                "ERROR".into(),
                f.metadata().await.unwrap(),
            )),
            Ok(None) => break,
            Err(e) => panic!("{}", e),
        }
    }

    // for file in walk {
    //     let file = file.unwrap();
    //     let metadata = file.metadata().unwrap();
    //     files.push(File::new(
    //         file.file_name().to_os_string().into_string().unwrap(),
    //         file.depth(),
    //         file.path(),
    //         file.parent_path().into(),
    //         metadata.into(),
    //     ));
    // }

    files

    // tx.send(Message::NewFiles(files)).await.unwrap();
}
