use crate::thread_pool::ThreadPool;
use crate::Result;
use failure::format_err;

pub struct RayonThreadPool {}

impl ThreadPool for RayonThreadPool {
    fn new(_threads: u32) -> Result<RayonThreadPool> {
        Err(format_err!("not implemented"))
    }
    fn spawn<F>(&self, _job: F)
    where
        F: FnOnce() + Send + 'static,
    {
    }
}
