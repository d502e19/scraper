use std::collections::HashSet;
use std::marker::PhantomData;

use url::Url;

use crate::task::Task;
use crate::traits::{Archive, Downloader, Extractor, Filter, Manager, Normaliser, TaskProcessResult};
use std::time::{SystemTime, UNIX_EPOCH};
use std::ops::Sub;
use crate::metrics::influx_client::{InfluxClient, get_timestamp_millis, add_data_point};
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
            // Only calculate start_time if log_process flag set high
            let task_start_time = get_timestamp_millis(enable_measuring);
            // Declare mutable points for aggregating measuring data
            let mut data_point: Point;
            // Do expensive setup if measuring is enabled, otherwise only setup a 'cheap' Point to avoid scope issues
            if enable_measuring {
                data_point = Point::new(format!("worker_processing_time").as_str())
                    .add_timestamp(task_start_time)
                    .add_field("start_time", Value::Integer(task_start_time))
                    .add_tag("instance", Value::String(self.name.clone())) // Fixme to be UUID
                    .to_owned()
            } else {
                data_point = Point::new("invalid_measurement");
            }

            info!("Worker {} received task {}", self.name, task.url);
            // Note that this function only mutates the point if 'enable' is high to minimise impact on non-measuring runs
            add_data_point(&mut data_point, "receive_task_time", task_start_time, enable_measuring);

            match self.downloader.fetch_page(&task) {
                Err(e) => {
                    error!("{} failed to download a page. {}", self.name, e);
                    return TaskProcessResult::from(e);
                }
                Ok(page) => {
                    add_data_point(&mut data_point, "download_task_time", task_start_time, enable_measuring);

                    match self.extractor.extract_content(page, &task.url) {
                        Err(e) => {
                            error!("{} failed to extract data from page. {}", self.name, e);
                            return TaskProcessResult::from(e);
                        }
                        Ok((mut urls, data)) => {
                            add_data_point(&mut data_point, "extract_task_time", task_start_time, enable_measuring);

                            // Archiving
                            if let Err(e) = self.archive.archive_content(data) {
                                error!("{} failed archiving some data. {}", self.name, e);
                                return TaskProcessResult::from(e);
                            }
                            add_data_point(&mut data_point, "archive_task_time", task_start_time, enable_measuring);

                            // Normalising urls
                            let tasks: Vec<Task> = self.normaliser.normalise(urls)
                                .drain(..)
                                .map(|url| Task { url })
                                .collect();

                            add_data_point(&mut data_point, "normalise_task_time", task_start_time, enable_measuring);

                            let filtered_tasks = self.filter.filter(tasks);

                            add_data_point(&mut data_point, "filter_task_time", task_start_time, enable_measuring);

                            // Cull tasks that have already been submitted once, then submit the new tasks
                            match self.manager.cull_known(filtered_tasks) {
                                Ok(new_tasks) => {
                                    add_data_point(&mut data_point, "culling_task_time", task_start_time, enable_measuring);

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
                            add_data_point(&mut data_point, "finishing_task_time", task_start_time, enable_measuring);
                            if enable_measuring {
                                // Write collected point of measuring data for given task and write to database through client
                                if let Some(client) = &influxdb_client {
                                    client.write_point(data_point.to_owned());
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

