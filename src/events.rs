use crossterm::event::{Event, EventStream, KeyEvent};
use futures::stream::StreamExt;
use tokio::sync::mpsc::Receiver;

pub struct Events {}

impl Events {
    pub fn start() -> Receiver<KeyEvent> {
        let (sx, rx) = tokio::sync::mpsc::channel(100);

        tokio::spawn(async move {
            let mut reader = EventStream::new();

            loop {
                let ev = reader.next().await;

                if let Some(Ok(Event::Key(event))) = ev {
                    sx.send(event).await.unwrap();
                }
            }
        });

        rx
    }
}
