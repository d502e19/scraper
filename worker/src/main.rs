extern crate futures;
extern crate lapin_futures;
extern crate rand;
extern crate redis;
extern crate tokio;

use std::collections::HashSet;
use std::error::Error;
use redis::Commands;
use url::Url;

use crate::downloader::DefaultDownloader;
use crate::task::Task;
use crate::traits::Downloader;

mod downloader;
mod extractor;
mod errors;
mod rmqredis;
mod split;
mod task;
mod traits;
mod void;
mod worker;
mod archive;

use crate::extractor::html::{HTMLExtractorBase, HTMLLinkExtractor};
use crate::rmqredis::RMQRedisManager;
use crate::void::Void;
use crate::worker::Worker;

fn main() -> Result<(), Box<dyn Error>> {
    // Construct a worker and its components
    let manager = RMQRedisManager::new(
        "192.168.99.100".to_string(),
        5672,
        6379,
        "work".to_string(),
        "".to_string(),
        "frontier".to_string(),
        "collection".to_string(),
    )
    .expect("Failed to construct RMQRedisManager");
    let downloader = DefaultDownloader;
    let extractor = HTMLExtractorBase::new(HTMLLinkExtractor::new());
    let archive = Void;
    let worker = Worker::new("W1".to_string(), manager, downloader, extractor, archive);
    worker.start();

    Ok(())
}
