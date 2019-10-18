use crate::traits::{Manager, Downloader, Extractor, Archive, TaskProcessResult};
use std::marker::PhantomData;

/// A worker is the web crawler module that resolves tasks. The components of the worker
/// define every aspect of the workers behaviour.
pub struct Worker<M, L, E, A, S, D> where
    M: Manager,
    L: Downloader<S>,
    E: Extractor<S, D>,
    A: Archive<D>,
{
    name: String,
    manager: M,
    downloader: L,
    extractor: E,
    archive: A,
    // Phantom data markers are used to please the type checker about S and D.
    // Without it will believe that S and D are unused even though the determine the
    // type parameters of some of the components
    _page_type_marker: PhantomData<S>,
    _data_type_marker: PhantomData<D>,
}

impl<M, L, E, A, S, D> Worker<M, L, E, A, S, D> where
    M: Manager,
    L: Downloader<S>,
    E: Extractor<S, D>,
    A: Archive<D>,
{
    /// Create a new worker with the given components.
    pub fn new(name: String, manager: M, downloader: L, extractor: E, archive: A) -> Self {
        Worker {
            name,
            manager,
            downloader,
            extractor,
            archive,
            _page_type_marker: PhantomData,
            _data_type_marker: PhantomData,
        }
    }

    /// Starts the worker. It will now listen to the manager for new tasks are resolve those.
    /// This is a blocking operation.
    pub fn start(&self) {
        self.manager.start_listening(|task| {
            println!("Worker {} received task {}", self.name, task.url);
            // TODO: Proper error handling
            match self.downloader.fetch_page(task) {
                Err(e) => {
                    eprintln!("{} failed to download a page.", self.name);
                    TaskProcessResult::Err
                }
                Ok(page) => {
                    match self.extractor.extract_content(page) {
                        Err(e) => {
                            eprintln!("{} failed to extract data from page.", self.name);
                            TaskProcessResult::Err
                        }
                        Ok((tasks, data)) => {
                            for datum in data {
                                if let Err(e) = self.archive.archive_content(datum) {
                                    eprintln!("{} failed archiving some data.", self.name);
                                }
                            }

                            for task in &tasks {
                                if let Ok(exists) = self.manager.contains(task) {
                                    if !exists {
                                        if let Err(_) = self.manager.submit_task(task) {
                                            eprintln!("{} failed submitting a new task to the manager.", self.name);
                                        }
                                    }
                                } else {
                                    eprintln!("{} failed to check if a new task is present in the collection. Skipping that task.", self.name)
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