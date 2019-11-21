use url::Url;

use crate::errors::{ArchiveResult, DownloadResult, ExtractResult, ManagerResult, NormaliseResult};
use crate::task::Task;

/// A Manager serves as the interface to the frontier and the collection
pub trait Manager {
    fn submit(&self, tasks: Vec<Task>) -> ManagerResult<()>;

    fn subscribe(&self, resolve_func: &dyn Fn(Task) -> TaskProcessResult);

    fn close(self) -> ManagerResult<()>;

    fn cull_known(&self, tasks: Vec<Task>) -> ManagerResult<Vec<Task>>;
}

/// A Frontier contains upcoming tasks
pub trait Frontier {
    fn submit(&self, task: Vec<Task>) -> ManagerResult<()>;

    fn subscribe(&self, resolve_func: &dyn Fn(Task) -> TaskProcessResult);

    fn close(self: Box<Self>) -> ManagerResult<()>;
}

/// When resolving a task, there are three different outcomes:
/// Ok: Task was completed
/// Err: Task was erroneous and should be discarded.
/// Reject: Task could not be completed this time. Let it be rescheduled.
pub enum TaskProcessResult {
    Ok,
    Err,
    Reject,
}

/// A Collection contains every found task, which prevents work duplications
pub trait Collection {
    fn cull_known(&self, tasks: Vec<Task>) -> ManagerResult<Vec<Task>>;

    fn submit(&self, tasks: Vec<Task>) -> ManagerResult<()>;

    fn close(self: Box<Self>) -> ManagerResult<()>;
}

/// The Downloader downloads the page S associated with the given task
pub trait Downloader<S> {
    fn fetch_page(&self, task: &Task) -> DownloadResult<S>;
}

/// The Extractor extracts new Urls and target data D from the page S
pub trait Extractor<S, D> {
    fn extract_content(&self, page: S, url: &Url) -> ExtractResult<(Vec<Url>, Vec<D>)>;
}

/// The Filter selects which tasks to visit. When the `filter` method returns true, the task should
/// be resolved.
pub trait Filter {
    fn filter(&self, tasks: Vec<Task>) -> Vec<Task>;
}

/// The Archive stores the target data D
pub trait Archive<D> {
    fn archive_content(&self, content: Vec<D>) -> ArchiveResult<()>;
}

/// The Normaliser normalises URLs to avoid different Urls to the same page
pub trait Normaliser {
    fn normalise(&self, url: Vec<Url>) -> Vec<Url>;
}
