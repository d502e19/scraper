use std::collections::HashSet;
use std::marker::PhantomData;

use url::Url;

use crate::task::Task;
use crate::traits::{Archive, Downloader, Extractor, Filter, Manager, Normaliser, TaskProcessResult};
use std::time::{SystemTime, UNIX_EPOCH};
use std::ops::Sub;
use crate::metrics::influx_client::{InfluxClient, get_timestamp_millis, MetricSession};
use influx_db_client::{Points, Point, Value};

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
    pub fn start(&self, enable_measuring: bool, influxdb_client: Option<InfluxClient>) {
        info!("Worker {} has started", self.name);
        self.manager.subscribe(&|task| {
            let mut metric_session = MetricSession::new("worker_processing_time", &self.name);

            info!("Worker {} received task {}", self.name, task.url);

            metric_session.add_data_point("receive_task_time");

            match self.downloader.fetch_page(&task) {
                Err(e) => {
                    error!("{} failed to download a page. {}", self.name, e);
                    return TaskProcessResult::from(e);
                }
                Ok(page) => {
                    metric_session.add_data_point("download_task_time");

                    match self.extractor.extract_content(page, &task.url) {
                        Err(e) => {
                            error!("{} failed to extract data from page. {}", self.name, e);
                            return TaskProcessResult::from(e);
                        }
                        Ok((mut urls, data)) => {
                            metric_session.add_data_point("extract_task_time");

                            // Archiving
                            if let Err(e) = self.archive.archive_content(data) {
                                error!("{} failed archiving some data. {}", self.name, e);
                                return TaskProcessResult::from(e);
                            }
                            metric_session.add_data_point("archive_task_time");

                            // Normalising urls
                            let tasks: Vec<Task> = self.normaliser.normalise(urls)
                                .drain(..)
                                .map(|url| Task { url })
                                .collect();

                            metric_session.add_data_point("normalise_task_time");

                            let filtered_tasks = self.filter.filter(tasks);

                            metric_session.add_data_point("filter_task_time");

                            // Cull tasks that have already been submitted once, then submit the new tasks
                            match self.manager.cull_known(filtered_tasks) {
                                Ok(new_tasks) => {
                                    metric_session.add_data_point("culling_task_time");

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

                            if let Some(client) = &influxdb_client {
                                metric_session.write_point(client);
                            }

                            return TaskProcessResult::Ok;
                        }
                    }
                }
            }
        });
    }
}

