use crate::thread_pool::ThreadPool;
use crate::Result;
use failure::format_err;

pub struct SharedQueueThreadPool {}

impl ThreadPool for SharedQueueThreadPool {
    fn new(_threads: u32) -> Result<SharedQueueThreadPool> {
        Err(format_err!("not implemented"))
    }
    fn spawn<F>(&self, _job: F)
    where
        F: FnOnce() + Send + 'static,
    {
    }
}
