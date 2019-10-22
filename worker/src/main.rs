extern crate futures;
extern crate lapin_futures;
extern crate rand;
extern crate redis;
extern crate tokio;

mod rmqredis;
mod split;
mod task;
mod traits;
mod downloader;
mod void;
mod extractor;
mod worker;

use crate::downloader::DefaultDownloader;
use crate::task::Task;
use crate::traits::Downloader;
use crate::worker::Worker;
use crate::rmqredis::RMQRedisManager;
use crate::void::Void;
use crate::extractor::html::{HTMLExtractorBase, HTMLLinkExtractor};
use std::collections::HashSet;
use std::error::Error;
use redis::Commands;

fn main() -> Result<(), Box<dyn Error>> {
    // Construct a worker and its components
    let manager = RMQRedisManager::new(
        "192.162.99.100".to_string(),
        5672,
        6379,
        "work".to_string(),
        "".to_string(),
        "frontier".to_string(),
        "collection".to_string(),
    ).expect("Failed to construct RMQRedisManager");
    let downloader = DefaultDownloader;
    let extractor = HTMLExtractorBase::new(HTMLLinkExtractor::new());
    let archive = Void;
    let worker = Worker::new(
        "Worker1".to_string(),
        manager,
        downloader,
        extractor,
        archive,
    );
    worker.start();

    println!("Hello from worker!");
    Ok(())
}
