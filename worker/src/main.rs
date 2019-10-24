extern crate clap;
extern crate futures;
extern crate lapin_futures;
extern crate rand;
extern crate redis;
extern crate tokio;

use std::collections::HashSet;
use std::error::Error;

use clap::{App, Arg};
use redis::Commands;

use crate::downloader::DefaultDownloader;
use crate::extractor::html::{HTMLExtractorBase, HTMLLinkExtractor};
use crate::rmqredis::RMQRedisManager;
use crate::task::Task;
use crate::traits::Downloader;
use crate::void::Void;
use crate::worker::Worker;

mod downloader;
mod extractor;
mod rmqredis;
mod split;
mod task;
mod traits;
mod void;
mod worker;

fn main() -> Result<(), Box<dyn Error>> {
    let args = App::new("DatScraper")
        .version("0.1.0")
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
            .short("p")
            .long("rmq-port")
            .value_name("PORT")
            .help("Specify the RabbitMQ port to connect to")
    ).arg(
        Arg::with_name("rabbitmq-exchange")
            .short("e")
            .long("rmq-exchange")
            .value_name("EXCHANGE")
            .help("Specify the RabbitMQ exchange to connect to")
    ).arg(
        Arg::with_name("rabbitmq-routing-key")
            .short("k")
            .long("rmq-routing-key")
            .value_name("KEY")
            .help("Specify the RabbitMQ routing-key to connect to")
    ).arg(
        Arg::with_name("rabbitmq-queue")
            .short("q")
            .long("rmq-queue")
            .value_name("QUEUE")
            .help("Specify the RabbitMQ queue to connect to")
    ).arg(
        Arg::with_name("redis-set")
            .short("s")
            .long("redis-set")
            .value_name("SET")
            .help("Specify the redis set to connect to")
    ).arg(
        Arg::with_name("manager-address")
            .short("a")
            .long("addr")
            .value_name("ADDR")
            .help("Specify the manager's address")
    ).get_matches();
    // Construct a worker and its components
    println!("{:?}", args.default_val)
    let manager = RMQRedisManager::new(
        args.value_of("manager-address").unwrap_or("localhost").to_string(),
        args.value_of("rabbitmq-port").unwrap_or("5672").parse().unwrap(),
        args.value_of("redis-port").unwrap_or("6379").parse().unwrap(),
        args.value_of("rabbitmq-exchange").unwrap_or("work").to_string(),
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
