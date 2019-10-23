extern crate clap;
extern crate futures;
extern crate lapin_futures;
extern crate rand;
extern crate redis;
extern crate tokio;

mod downloader;
mod extractor;
mod rmqredis;
mod split;
mod task;
mod traits;
mod void;
mod worker;

use crate::downloader::DefaultDownloader;
use crate::extractor::html::{HTMLExtractorBase, HTMLLinkExtractor};
use crate::rmqredis::RMQRedisManager;
use crate::task::Task;
use crate::traits::Downloader;
use crate::void::Void;
use crate::worker::Worker;
use clap::{App, Arg};
use redis::Commands;
use std::collections::HashSet;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let args = App::new("DatScraper")
        .version("0.1")
        .author("d502e19@aau")
        .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .value_name("FILE")
                .help("Specify a config file to use"),
        ).arg(
        Arg::with_name("redis-port")
            .short("r")
            .long("redis-port")
            .value_name("PORT")
            .help("Specify the redis-port to connect to")
    ).arg(
        Arg::with_name("rabbitmq-port")
            .short("q")
            .long("rmq-port")
            .value_name("PORT")
            .help("Specify the RabbitMQ-port to connect to")
    ).arg(
        Arg::with_name("manager-address")
            .short("a")
            .long("addr")
            .value_name("ADDR")
            .help("Specify the manager's address")
    ).get_matches();
    // Construct a worker and its components
    let manager = RMQRedisManager::new(
        args.value_of("manager-address").unwrap_or("localhost").to_string(),
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
    let worker = Worker::new("W1".to_string(), manager, downloader, extractor, archive);
    worker.start();

    Ok(())
}
