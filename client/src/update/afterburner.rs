use std::sync::{
    atomic::{AtomicU32, Ordering},
    Arc,
};
use tokio::sync::mpsc;

use super::download::{Download, DownloadError};

type AfterburnerItem<T> = Result<Download<T>, DownloadError>;

/// Downloads need quite some long time for the initial .get call, so doing them
/// sequentially is slow.
/// Slap an Afterburner on it.
#[derive(Debug)]
pub(super) struct Afterburner<T> {
    in_queue: Arc<AtomicU32>,
    receiver: mpsc::Receiver<AfterburnerItem<T>>,
    sender: mpsc::Sender<AfterburnerItem<T>>,
}

impl<T: Send + 'static> Afterburner<T> {
    pub fn new(parallel: usize) -> Self {
        let (sender, receiver) = mpsc::channel(parallel * 2);
        Self {
            in_queue: Arc::new(AtomicU32::new(0)),
            receiver,
            sender,
        }
    }

    pub(super) fn len(&self) -> u32 {
        self.in_queue.load(Ordering::SeqCst)
    }

    pub(super) fn next(&mut self) -> Option<AfterburnerItem<T>> {
        self.receiver.try_recv().ok()
    }

    pub(super) async fn start(&self, d: Download<T>) {
        let in_queue = self.in_queue.clone();
        in_queue.fetch_add(1, Ordering::SeqCst);
        let sender = self.sender.clone();
        tokio::spawn(async move {
            let open_connection = d.progress().await;
            let _ = sender.send(open_connection).await;
            in_queue.fetch_sub(1, Ordering::SeqCst);
        });
    }
}
