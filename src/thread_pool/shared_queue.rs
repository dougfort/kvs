use failure::{format_err};
use crate::Result;
use crate::thread_pool::ThreadPool;

pub struct SharedQueueThreadPool {}

impl ThreadPool for SharedQueueThreadPool {
    fn new(_threads: u32) -> Result<SharedQueueThreadPool> {
        Err(format_err!("not implemented")) 
    }
    fn spawn<F>(&self, _job: F) where F: FnOnce() + Send + 'static{
        ()
    }
}

