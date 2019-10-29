use std::marker::PhantomData;

use crate::traits::{Archive, Downloader, Extractor, Manager, TaskProcessResult, Filter};

/// A worker is the web crawler module that resolves tasks. The components of the worker
/// define every aspect of the workers behaviour.
pub struct Worker<M, L, E, A, S, D, F>
where
    M: Manager,
    L: Downloader<S>,
    E: Extractor<S, D>,
    A: Archive<D>,
    F: Filter<F>,
{
    name: String,
    manager: M,
    downloader: L,
    extractor: E,
    archive: A,
    filter: F,
    // Phantom data markers are used to please the type checker about S and D.
    // Without it will believe that S and D are unused even though the determine the
    // type parameters of some of the components
    _page_type_marker: PhantomData<S>,
    _data_type_marker: PhantomData<D>,
}

impl<M, L, E, A, S, D, F> Worker<M, L, E, A, S, D, F>
where
    M: Manager,
    L: Downloader<S>,
    E: Extractor<S, D>,
    A: Archive<D>,
    F: Filter<F>,
{
    /// Create a new worker with the given components.
    pub fn new(name: String, manager: M, downloader: L, extractor: E, archive: A, filter: F) -> Self {
        Worker {
            name,
            manager,
            downloader,
            extractor,
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
        println!("Worker {} has started", self.name);
        self.manager.start_listening(move |task| {
            println!("Worker {} received task {}", self.name, task.url);
            // TODO: Proper error handling
            match self.downloader.fetch_page(&task) {
                Err(e) => {
                    eprintln!("{} failed to download a page.", self.name);
                    TaskProcessResult::Err
                }
                Ok(page) => {
                    match self.extractor.extract_content(page, &task.url) {
                        Err(e) => {
                            eprintln!("{} failed to extract data from page.", self.name);
                            TaskProcessResult::Err
                        }
                        Ok((tasks, data)) => {
                            // Archiving
                            for datum in data {
                                if let Err(e) = self.archive.archive_content(datum) {
                                    eprintln!("{} failed archiving some data.", self.name);
                                    return TaskProcessResult::Err;
                                }
                            }

                            // Check if extracted links are new, if they are, submit them
                            for task in &tasks {
                                if self.filter.filter(task) {
                                    if let Ok(exists) = self.manager.contains(task) {
                                        if !exists {
                                            if let Err(_) = self.manager.submit_task(task) {
                                                eprintln!("{} failed submitting a new task to the manager.", self.name);
                                                return TaskProcessResult::Err;
                                            }
                                        }
                                    } else {
                                        eprintln!("{} failed to check if a new task is present in the collection. Ignoring that task.", self.name);
                                        return TaskProcessResult::Err;
                                    }
                                } else {
                                    eprintln!("{} url in task is not in the whitelist.txt file. Ignoring that task", self.name);
                                    return TaskProcessResult::Err;
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
