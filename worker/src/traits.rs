use crate::task::Task;
use std::error::Error;

pub(crate) trait FrontierSubmitted {
    fn submit_task(&self, task: &Task) -> Result<(), ()>;

    fn start_listening<F>(&self, f: F)
    where
        F: Fn(&Task) -> TaskProcessResult;

    fn close(self) -> Result<(), ()>;

    fn contains(&self, task: &Task) -> Result<bool, ()>;
}

pub trait Frontier {
    fn submit_task(&self, task: &Task) -> Result<(), ()>;

    fn start_listening<F>(&self, f: F)
    where
        F: Fn(&Task) -> TaskProcessResult;

    fn close(self) -> Result<(), ()>;
}

pub enum TaskProcessResult {
    Ok,
    Err,
    Reject,
}

pub trait Submitted {
    fn contains(&self, task: &Task) -> Result<bool, ()>;

    fn submit_task(&self, task: &Task) -> Result<(), ()>;
}

pub trait Downloader<S> {
    fn fetch_page(&self, task: Task) -> Result<S, Box<dyn Error>>;
}

pub trait Extractor<S, D> {
    fn extract_content(&self, page: S) -> Result<(Vec<Task>, Vec<D>), Box<dyn Error>>;
}

pub trait Archive<D> {
    fn archive_content(&self, content: D) -> Result<(), Box<dyn Error>>;
}