use crate::traits::{Manager, Downloader, Extractor, Archive, TaskProcessResult};
use std::marker::PhantomData;

struct Worker<M, L, E, A, S, D> where
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
    _page_type_marker: PhantomData<S>,
    _data_type_marker: PhantomData<D>,
}

impl<M, L, E, A, S, D> Worker<M, L, E, A, S, D> where
    M: Manager,
    L: Downloader<S>,
    E: Extractor<S, D>,
    A: Archive<D>,
{
    fn new(name: String, manager: M, downloader: L, extractor: E, archive: A) -> Self {
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

    fn work(&self) {
        self.manager.start_listening(|task| {
            println!("Worker {} received task {}", self.name, task.url);

            match self.downloader.fetch_page(task) {
                Err(e) => {
                    TaskProcessResult::Err
                }
                Ok(page) => {
                    match self.extractor.extract_content(page) {
                        Err(e) => {
                            TaskProcessResult::Err
                        }
                        Ok((tasks, data)) => {
                            for datum in data {
                                self.archive.archive_content(datum);
                            }

                            for task in &tasks {
                                if let Ok(exists) = self.manager.contains(task) {
                                    if !exists {
                                        self.manager.submit_task(task);
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