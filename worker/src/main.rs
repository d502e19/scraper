extern crate clap;
extern crate futures;
extern crate lapin_futures;
#[macro_use]
extern crate log;
extern crate log4rs;
extern crate rand;
extern crate redis;
extern crate tokio;

use std::error::Error;
use std::io::ErrorKind;

use clap::{App, AppSettings, Arg, ArgMatches};
use log4rs::append::console::ConsoleAppender;
use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Config, Logger, Root};
use log4rs::encode::pattern::PatternEncoder;
use log::Level;
use log::LevelFilter;

use crate::defaultnormaliser::DefaultNormaliser;
use crate::downloader::DefaultDownloader;
use crate::extractor::html::{HTMLExtractorBase, HTMLLinkExtractor};
use crate::filter::filter::Whitelist;
use crate::rmqredis::RMQRedisManager;
use crate::task::Task;
use crate::void::Void;
use crate::worker::Worker;

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

/// Create and return log4rs-config with some default values
fn get_log4rs_config(log_path: &str, default_log_level: LevelFilter) -> log4rs::config::Config {
    // Create a stdout-appender for printing to stdout
    let stdout = ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{d} [{l}] {t} - {m}{n}")))
        .build();

    // Create a logfile-appender for printing to file
    let logfile = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{d} [{l}] {t} - {m}{n}")))
        .build(log_path)
        .unwrap();

    // Create and return a config which incorporates the two built appenders
    // and let both appenders be root loggers with 'info' as log-level
    Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .appender(Appender::builder().build("logfile", Box::new(logfile)))
        .build(Root::builder().appender("stdout").appender("logfile").build(default_log_level))
        .unwrap()
}

fn main() -> Result<(), Box<dyn Error>> {
    // Set up arguments and get resulting arguments
    let args = App::new("DatScraper Worker")
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
    ).arg (
        Arg::with_name("log-path")
            .short("l")
            .long("log-path")
            .env("LOG_PATH")
            .default_value("worker.log")
            .value_name("PATH")
            .help("Specify the log-file path")
    ).arg(
        Arg::with_name("log-level")
            .short("o")
            .long("log-level")
            .env("LOG_LEVEL")
            .default_value("info")
            .value_name("LEVEL")
            .help("Specify the log level {error, warn, info, debug, trace, off}")
    ).arg(
        Arg::with_name("filter-enable")
            .short("f")
            .long("filter-enable")
            .env("SCRAPER_FILTER_ENABLE")
            .default_value("false")
            .value_name("BOOLEAN")
            .help("Specify whether filtering is enabled")
    ).arg(
        Arg::with_name("filter-path")
            .short("w")
            .long("filter-path")
            .env("SCRAPER_FILTER_PATH")
            .default_value("src/filter/whitelist.txt")
            .value_name("PATH")
            .help("Specify path to list for filtering")
    ).arg(
        Arg::with_name("filter-type")
            .short("t")
            .long("filter-type")
            .env("SCRAPER_FILTER_TYPE")
            .default_value("white")
            .value_name("STRING")
            .help("Specify whether the list in the given filter-path is a 'white' or 'black'-list")
    ).get_matches();

    // Load config for logging to stdout and logfile.
    if let Ok(handle) = log4rs::init_config(
        get_log4rs_config(
            args.value_of("log-path").unwrap(),
            match args.value_of("log-level").unwrap().to_lowercase().as_str() {
                "error" => LevelFilter::Error,
                "warn" => LevelFilter::Warn,
                "info" => LevelFilter::Info,
                "debug" => LevelFilter::Debug,
                "trace" => LevelFilter::Trace,
                "off" => LevelFilter::Off,
                _ => LevelFilter::Off,
            })
    ) {
        info!("Starting worker module using RabbitMQ and redis on {:?}",
              args.value_of("manager-address").unwrap().to_string());

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
        let filter = Whitelist::new(
            args.value_of("filter-path").unwrap().to_string(),
            args.value_of("filter-enable").unwrap().parse().unwrap());
        let normaliser = DefaultNormaliser;
        let archive = Void;
        let worker = Worker::new(
            "W1",
            Box::new(manager),
            Box::new(downloader),
            Box::new(extractor),
            Box::new(normaliser),
            Box::new(archive),
            Box::new(filter),
        );
        worker.start();

        Ok(())
    } else {
        Err(Box::new(std::io::Error::new(ErrorKind::Other, "[ERROR] Failed creating logging config")))
    }
}