use flume::{unbounded, Sender};
use std::collections::VecDeque;
use std::sync::{Arc, RwLock};

pub type Queue<T> = Arc<RwLock<VecDeque<T>>>;

pub fn empty_queue<T: Send + Sync + 'static>() -> Queue<T> {
    Default::default()
}

pub fn start_queue_thread<T: Send + Sync + 'static>(queue: Queue<T>) -> Sender<T> {
    let (tx_request, rx_request) = unbounded();
    std::thread::spawn(move || {
        let mut temp_queue = VecDeque::new();
        loop {
            if !temp_queue.is_empty() {
                if let Ok(mut queue) = queue.try_write() {
                    while let Some(req) = temp_queue.pop_front() {
                        queue.push_back(req);
                    }
                }
            }
            if let Ok(gen_image_request) = rx_request.try_recv() {
                if let Ok(mut queue) = queue.try_write() {
                    queue.push_back(gen_image_request);
                } else {
                    temp_queue.push_back(gen_image_request);
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
    });

    tx_request
}
