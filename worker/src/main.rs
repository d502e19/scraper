extern crate futures;
extern crate lapin_futures;
extern crate rand;
extern crate redis;
extern crate tokio;

use std::collections::HashSet;
use std::error::Error;

use redis::Commands;

use crate::downloader::DefaultDownloader;
use crate::extractor::html::{HTMLExtractorBase, HTMLLinkExtractor};
use crate::rmqredis::RMQRedisManager;
use crate::task::Task;
use crate::traits::Downloader;
use crate::void::Void;
use crate::worker::Worker;
use crate::filter::filter::Whitelist;

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
mod defaultnormaliser;
mod filter;

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
    let downloader = DefaultDownloader::new();
    let extractor = HTMLExtractorBase::new(HTMLLinkExtractor::new());
    let filter = Whitelist::new();
    let archive = Void;
    let worker = Worker::new("W1".to_string(), manager, downloader, extractor, archive, filter);
    worker.start();

    Ok(())
}