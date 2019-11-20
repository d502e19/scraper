use std::collections::HashSet;
use std::marker::PhantomData;
use url::Url;

use crate::traits::{Archive, Downloader, Extractor, Manager, TaskProcessResult, Normaliser, Filter};
use crate::task::Task;

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
        self.manager.start_listening(&|task| {
            info!("Worker {} received task {}", self.name, task.url);
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
                            for datum in data {
                                if let Err(e) = self.archive.archive_content(datum) {
                                    error!("{} failed archiving some data. {}", self.name, e);
                                    return TaskProcessResult::from(e);
                                }
                            }

                            // Normalise extracted links
                            // After normalisation, squash urls into a hash set to remove duplicates
                            // Erroneous urls are discarded
                            let tasks: Vec<Task> = self.normaliser.normalise(urls)
                                .drain(..)
                                .map(|url| Task { url })
                                .collect();


                            // Check if extracted tasks are new, if they are, submit them
                            for task in &tasks {
                                if self.filter.filter(&task) {
                                    match self.manager.contains(task) {
                                        Ok(exists) => {
                                            if !exists {
                                                if let Err(e) = self.manager.submit_task(task) {
                                                    error!("{} failed submitting a new task to the manager. {}", self.name, e);
                                                    return TaskProcessResult::from(e);
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            error!("{} failed to check if a new task is present in the collection. {}", self.name, e);
                                            return TaskProcessResult::from(e);
                                        }
                                    }
                                }
                            }

                            return TaskProcessResult::Ok;
                        }
                    }
                }
            }
        });
    }
}

