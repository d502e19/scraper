use std::collections::HashSet;
use std::marker::PhantomData;

use url::Url;

use crate::task::Task;
use crate::traits::{Archive, Downloader, Extractor, Manager, Normaliser, TaskProcessResult};

/// A worker is the web crawler module that resolves tasks. The components of the worker
/// define every aspect of the workers behaviour.
pub struct Worker<M, L, E, N, A, S, D>
    where
        M: Manager,
        L: Downloader<S>,
        E: Extractor<S, D>,
        N: Normaliser,
        A: Archive<D>,
{
    name: String,
    manager: M,
    downloader: L,
    extractor: E,
    normaliser: N,
    archive: A,
    // Phantom data markers are used to please the type checker about S and D.
    // Without it will believe that S and D are unused even though the determine the
    // type parameters of some of the components
    _page_type_marker: PhantomData<S>,
    _data_type_marker: PhantomData<D>,
}

impl<M, L, E, N, A, S, D> Worker<M, L, E, N, A, S, D>
    where
        M: Manager,
        L: Downloader<S>,
        E: Extractor<S, D>,
        N: Normaliser,
        A: Archive<D>,
{
    /// Create a new worker with the given components.
    pub fn new(name: String, manager: M, downloader: L, extractor: E, normaliser: N, archive: A) -> Self {
        Worker {
            name,
            manager,
            downloader,
            extractor,
            normaliser,
            archive,
            _page_type_marker: PhantomData,
            _data_type_marker: PhantomData,
        }
    }

    /// Starts the worker. It will now listen to the manager for new tasks are resolve those.
    /// Resolving includes downloading, extracting, archiving, and submitting new tasks.
    /// This is a blocking operation.
    pub fn start(&self) {
        info!("Worker {} has started", self.name);
        self.manager.start_listening(move |task| {
            info!("Worker {} received task {}", self.name, task.url);
            match self.downloader.fetch_page(&task) {
                Err(e) => {
                    error!("{} failed to download a page. {}", self.name, e);
                    TaskProcessResult::Err
                }
                Ok(page) => {
                    match self.extractor.extract_content(page, &task.url) {
                        Err(e) => {
                            error!("{} failed to extract data from page. {}", self.name, e);
                            TaskProcessResult::Err
                        }
                        Ok((mut urls, data)) => {
                            // Archiving
                            for datum in data {
                                if let Err(e) = self.archive.archive_content(datum) {
                                    error!("{} failed archiving some data. {}", self.name, e);
                                    return TaskProcessResult::Err;
                                }
                            }

                            // Normalise extracted links
                            // After normalisation, squash urls into a hash set to remove duplicates
                            // Erroneous urls are discarded
                            let tasks: Vec<Task> = urls.drain(..)
                                .filter_map(|url| {
                                    let url_as_str = String::from(url.as_str());
                                    match self.normaliser.normalise(url) {
                                        Ok(normalised_url) => Some(normalised_url),
                                        Err(e) => {
                                            eprintln!("{} failed to normalise {}, {}", self.name, url_as_str, e);
                                            None
                                        }
                                    }
                                })
                                .collect::<HashSet<Url>>()
                                .drain()
                                .map(|url| Task { url })
                                .collect();

                            // Check if extracted tasks are new, if they are, submit them
                            for task in &tasks {
                                match self.manager.contains(task) {
                                    Ok(exists) => {
                                        if !exists {
                                            if let Err(e) = self.manager.submit_task(task) {
                                                error!("{} failed submitting a new task to the manager. {}", self.name, e);
                                                return TaskProcessResult::Err;
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        error!("{} failed to check if a new task is present in the collection. Ignoring that task. {}", self.name, e);
                                        return TaskProcessResult::Err;
                                    }
                                }
                            }

                            TaskProcessResult::Ok
                        }
                    }
                }
            }
        });
    }
}
