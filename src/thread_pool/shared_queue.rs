use crate::thread_pool::ThreadPool;
use crate::Result;
use crossbeam_channel::{bounded, Sender, Receiver};
use std::thread;

enum ThreadPoolMessage {
    RunJob(Box<FnOnce() + Send + 'static>),
	Shutdown,
}

pub struct SharedQueueThreadPool {
    count: u32,
    sender: Sender<ThreadPoolMessage>,
 }

impl ThreadPool for SharedQueueThreadPool {
    fn new(threads: u32) -> Result<SharedQueueThreadPool> {
        let (sender, receiver) = bounded(threads as usize);
        for _n in 1..threads {
            let thread_receiver = receiver.clone();
            thread::spawn(move || {
                thread_launcher(thread_receiver);
            });
        };
        Ok(SharedQueueThreadPool{count: threads, sender: sender})
    }
    fn spawn<F>(&self, job: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.sender.send(ThreadPoolMessage::RunJob(Box::new(job))).expect("send failed");
    }
}

fn thread_launcher(receiver: Receiver<ThreadPoolMessage>) {
    loop {
        let thread_receiver = receiver.clone();
        let join_handle = thread::spawn(move || {
            loop {
                match thread_receiver.recv().expect("recv failed") {
                    ThreadPoolMessage::RunJob(job) => {
                        job();
                    },
                    ThreadPoolMessage::Shutdown => break
                };
             };
        });
        // if the thread stops normally, (not panic), we are shutting down
        if join_handle.join().is_ok() {
            break
        }
    }
}

impl Drop for SharedQueueThreadPool {
    fn drop(&mut self) {
        if !thread::panicking() {
            for _n in 1..self.count {
                self.sender.send(ThreadPoolMessage::Shutdown).expect("ThreadPoolMessage::Shutdown");
            }
        }
    }
}

