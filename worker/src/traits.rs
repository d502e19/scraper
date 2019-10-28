use std::error::Error;

use url::Url;

use crate::errors::{ArchiveResult, DownloadResult, ExtractResult, ManagerResult};
use crate::task::Task;

pub trait Manager {
    fn submit_task(&self, task: &Task) -> ManagerResult<()>;

    fn start_listening<F>(&self, f: F)
        where
        F: Fn(Task) -> TaskProcessResult;

    fn close(self) -> ManagerResult<()>;

    fn contains(&self, task: &Task) -> ManagerResult<bool>;
}

pub trait Frontier {
    fn submit_task(&self, task: &Task) -> ManagerResult<()>;

    fn start_listening<F>(&self, f: F)
        where
        F: Fn(Task) -> TaskProcessResult;

    fn close(self) -> ManagerResult<()>;
}

pub enum TaskProcessResult {
    Ok,
    Err,
    Reject,
}

pub trait Collection {
    fn contains(&self, task: &Task) -> ManagerResult<bool>;

    fn submit_task(&self, task: &Task) -> ManagerResult<()>;
}

pub trait Downloader<S> {
    fn fetch_page(&self, task: &Task) -> DownloadResult<S>;
}

pub trait Extractor<S, D> {
    fn extract_content(&self, page: S, url: &Url) -> ExtractResult<(Vec<Task>, Vec<D>)>;
}

pub trait Archive<D> {
    fn archive_content(&self, content: D) -> ArchiveResult<()>;
}

pub trait Normaliser {
    fn normalise(&self, task: Task) -> Result<Task, Box<dyn Error>>;
}
