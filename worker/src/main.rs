extern crate clap;
extern crate futures;
extern crate lapin_futures;
extern crate rand;
extern crate redis;
extern crate tokio;

use std::error::Error;

use clap::{App, Arg};

use crate::downloader::DefaultDownloader;
use crate::extractor::html::{HTMLExtractorBase, HTMLLinkExtractor};
use crate::rmqredis::RMQRedisManager;
use crate::task::Task;
use crate::void::Void;
use crate::worker::Worker;
use crate::defaultnormaliser::DefaultNormaliser;

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

fn main() -> Result<(), Box<dyn Error>> {
    // Set up arguments and get resulting arguments
    let args = App::new("DatScraper")
        .version("0.1.0")
        .author("d502e19@aau")
        .arg(
            Arg::with_name("manager-address")
                .short("a")
                .long("addr")
                .env("SCRAPER_MANAGER_ADDRESS")
                // Checks for system at compile-time, not runtime
                .default_value(if cfg!(windows) { "192.168.99.100" } else { "localhost" })
                .value_name("ADDR")
                .help("Specify the manager's address")
        ).arg(
        Arg::with_name("rabbitmq-port")
            .short("p")
            .long("rmq-port")
            .env("SCRAPER_RABBITMQ_PORT")
            .default_value("5672")
            .value_name("PORT")
            .help("Specify the RabbitMQ port to connect to")
    ).arg(
        Arg::with_name("redis-port")
            .short("r")
            .long("redis-port")
            .env("SCRAPER_REDIS_PORT")
            .default_value("6379")
            .value_name("PORT")
            .help("Specify the redis-port to connect to")
    ).arg(
        Arg::with_name("rabbitmq-exchange")
            .short("e")
            .long("rmq-exchange")
            .env("SCRAPER_RABBITMQ_EXCHANGE")
            .default_value("work")
            .value_name("EXCHANGE")
            .help("Specify the RabbitMQ exchange to connect to")
    ).arg(
        Arg::with_name("rabbitmq-routing-key")
            .short("k")
            .long("rmq-routing-key")
            .env("SCRAPER_RABBITMQ_ROUTING_KEY")
            .default_value("") // No routing-key by default
            .value_name("KEY")
            .help("Specify the RabbitMQ routing-key to connect to")
    ).arg(
        Arg::with_name("rabbitmq-queue")
            .short("q")
            .long("rmq-queue")
            .env("SCRAPER_RABBITMQ_QUEUE")
            .default_value("frontier")
            .value_name("QUEUE")
            .help("Specify the RabbitMQ queue to connect to")
    ).arg(
        Arg::with_name("redis-set")
            .short("s")
            .long("redis-set")
            .env("SCRAPER_REDIS_SET")
            .default_value("collection")
            .value_name("SET")
            .help("Specify the redis set to connect to")
    ).get_matches();

    // Construct a worker and its components
    let manager = RMQRedisManager::new(
        args.value_of("manager-address").unwrap().to_string(),
        args.value_of("rabbitmq-port").unwrap().parse().unwrap(), // Parse str to u16
        args.value_of("redis-port").unwrap().parse().unwrap(), // Parse str to u16
        args.value_of("rabbitmq-exchange").unwrap().to_string(),
        args.value_of("rabbitmq-routing-key").unwrap().to_string(),
        args.value_of("rabbitmq-queue").unwrap().to_string(),
        args.value_of("redis-set").unwrap().to_string(),
    ).expect("Failed to construct RMQRedisManager");
    let downloader = DefaultDownloader::new();
    let extractor = HTMLExtractorBase::new(HTMLLinkExtractor::new());
    let normaliser = DefaultNormaliser;
    let archive = Void;
    let worker = Worker::new("W1".to_string(), manager, downloader, extractor, normaliser, archive);
    worker.start();

    Ok(())
}