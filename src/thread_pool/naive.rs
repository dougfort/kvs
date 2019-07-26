use failure::{format_err};
use crate::Result;
use crate::thread_pool::ThreadPool;

pub struct NaiveThreadPool {}

impl ThreadPool for NaiveThreadPool {
    fn new(_threads: u32) -> Result<NaiveThreadPool> {
        Err(format_err!("not implemented")) 
    }
    fn spawn<F>(&self, _job: F) where F: FnOnce() + Send + 'static{
        ()
    }
}

