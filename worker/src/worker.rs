use std::collections::HashSet;
use std::marker::PhantomData;

use url::Url;

use crate::task::Task;
use crate::traits::{Archive, Downloader, Extractor, Filter, Manager, Normaliser, TaskProcessResult};
use std::time::{SystemTime, UNIX_EPOCH};
use std::ops::Sub;

/// A worker is the web crawler module that resolves tasks. The components of the worker
/// define every aspect of the workers behaviour.
pub struct Worker<S, D> {
    name: String,
    manager: Box<dyn Manager>,
    downloader: Box<dyn Downloader<S>>,
    extractor: Box<dyn Extractor<S, D>>,
    normaliser: Box<dyn Normaliser>,
    archive: Box<dyn Archive<D>>,
    filter: Box<dyn Filter>,

    // Phantom data markers are used to please the type checker about S and D.
    // Without it will believe that S and D are unused even though the determine the
    // type parameters of some of the components
    _page_type_marker: PhantomData<S>,
    _data_type_marker: PhantomData<D>,
}

impl<S, D> Worker<S, D> {
    /// Create a new worker with the given components.
    pub fn new(
        name: &str,
        manager: Box<dyn Manager>,
        downloader: Box<dyn Downloader<S>>,
        extractor: Box<dyn Extractor<S, D>>,
        normaliser: Box<dyn Normaliser>,
        archive: Box<dyn Archive<D>>,
        filter: Box<dyn Filter>,
    ) -> Self {
        Worker {
            name: String::from(name),
            manager,
            downloader,
            extractor,
            normaliser,
            archive,
            filter,
            _page_type_marker: PhantomData,
            _data_type_marker: PhantomData,
        }
    }

    /// Starts the worker. It will now listen to the manager for new tasks are resolve those.
    /// Resolving includes downloading, extracting, archiving, and submitting new tasks.
    /// This is a blocking operation.
    pub fn start(&self) {
        info!("Worker {} has started", self.name);
        self.manager.subscribe(&|task| {
            let task_start_time = match SystemTime::now().duration_since(UNIX_EPOCH) {
                Ok(time) => { time.as_millis() }
                Err(e) => {
                    error!("Could not get system time during receiving task on worker {} and task {}",
                           self.name,
                           task.url);
                    // Return zero if no time could be found to avoid breaking entire worker
                    0
                }
            };
            info!("Worker {} received task {} at unixtime:{:?}ms", self.name, task.url, task_start_time);
            match self.downloader.fetch_page(&task) {
                Err(e) => {
                    error!("{} failed to download a page. {}", self.name, e);
                    return TaskProcessResult::from(e);
                }
                Ok(page) => {
                    match self.extractor.extract_content(page, &task.url) {
                        Err(e) => {
                            error!("{} failed to extract data from page. {}", self.name, e);
                            return TaskProcessResult::from(e);
                        }
                        Ok((mut urls, data)) => {
                            // Archiving
                            if let Err(e) = self.archive.archive_content(data) {
                                error!("{} failed archiving some data. {}", self.name, e);
                                return TaskProcessResult::from(e);
                            }

                            // Normalising urls
                            let tasks: Vec<Task> = self.normaliser.normalise(urls)
                                .drain(..)
                                .map(|url| Task { url })
                                .collect();

                            let filtered_tasks = self.filter.filter(tasks);

                            // Cull tasks that have already been submitted once, then submit the new tasks
                            match self.manager.cull_known(filtered_tasks) {
                                Ok(new_tasks) => {
                                    if let Err(e) = self.manager.submit(new_tasks) {
                                        error!("{} failed submitting new tasks to the manager. {}", self.name, e);
                                        return TaskProcessResult::from(e);
                                    }
                                }
                                Err(e) => {
                                    error!("{} failed to check if tasks are present in the collection. {}", self.name, e);
                                    return TaskProcessResult::from(e);
                                }
                            }
                            let finishing_time = match SystemTime::now().duration_since(UNIX_EPOCH) {
                                Ok(time) => {
                                    time.as_millis().sub(task_start_time)
                                }
                                Err(e) => {
                                    error!("Could not get system time during receiving task on worker {} and task {}",
                                           self.name,
                                           task.url);
                                    // Return zero if no time could be found to avoid breaking entire worker
                                    0
                                }
                            };

                            info!("Worker {} finished task {} in {:?}ms", self.name, task.url, finishing_time);
                            return TaskProcessResult::Ok;
                        }
                    }
                }
            }
        });
    }
}

