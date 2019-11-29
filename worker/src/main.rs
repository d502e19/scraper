extern crate clap;
extern crate futures;
extern crate lapin_futures;
#[macro_use]
extern crate log;
extern crate log4rs;
extern crate rand;
extern crate redis;
extern crate tokio;

#[macro_use]
extern crate influx_db_client;

use std::error::Error;
use std::io::ErrorKind;

use clap::{App, Arg};
use log::LevelFilter;
use log4rs::append::console::ConsoleAppender;
use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Config, Root};
use log4rs::encode::pattern::PatternEncoder;

use crate::defaultnormaliser::DefaultNormaliser;
use crate::downloader::DefaultDownloader;
use crate::extractor::html::{HTMLExtractorBase, HTMLLinkExtractor};
use crate::filter::filter::{Blacklist, NoFilter, Whitelist};
use crate::metrics::influx_client::InfluxClient;
use crate::rmqredis::RMQRedisManager;
use crate::task::Task;
use crate::traits::Filter;
use crate::void::Void;
use crate::worker::Worker;
use std::time::SystemTime;

mod archive;
mod defaultnormaliser;
mod downloader;
mod errors;
mod extractor;
mod filter;
mod metrics;
mod rmqredis;
mod split;
mod task;
mod traits;
mod void;
mod worker;

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
        .build(
            Root::builder()
                .appender("stdout")
                .appender("logfile")
                .build(default_log_level),
        )
        .unwrap()
}

fn main() -> Result<(), Box<dyn Error>> {
    // Set up arguments and get resulting arguments
    let args = App::new("DatScraper Worker")
        .version("0.1.0")
        .author("d502e19@aau")
        .arg(
            Arg::with_name("rmq-address")
                .short("a")
                .long("rmq-addr")
                .env("SCRAPER_RMQ_ADDRESS")
                // Checks for system at compile-time, not runtime
                .default_value(if cfg!(windows) { "192.168.99.100" } else { "localhost" })
                .value_name("ADDR")
                .help("Specify the RabbitMQ address")
        ).arg(
        Arg::with_name("redis-address")
            .short("b")
            .long("redis-addr")
            .env("SCRAPER_REDIS_ADDRESS")
            // Checks for system at compile-time, not runtime
            .default_value(if cfg!(windows) { "192.168.99.100" } else { "localhost" })
            .value_name("ADDR")
            .help("Specify the Redis address")
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
            Arg::with_name("rabbitmq-prefetch-count")
                .short("n")
                .long("rmq-prefetch-count")
                .env("SCRAPER_RABBITMQ_PREFETCH_COUNT")
                .default_value("5")
                .value_name("COUNT")
                .help("Specify the number of tasks to prefetch")
        ).arg(
            Arg::with_name("rabbitmq-queue")
                .short("q")
                .long("rmq-queue")
                .env("SCRAPER_RABBITMQ_QUEUE")
                .default_value("frontier")
                .value_name("QUEUE")
                .help("Specify the RabbitMQ queue to connect to")
        ).arg(
            Arg::with_name("rabbitmq-collection-queue")
                .short("c")
                .long("rmq-collection")
                .env("SCRAPER_RABBITMQ_COLLECTION_QUEUE")
                .default_value("collection")
                .value_name("COLLECTION")
                .help("Specify the RabbitMQ collection queue to connect to")
        ).arg(
            Arg::with_name("sentinel")
                .short("m")
                .long("sentinel")
                .env("SCRAPER_SENTINEL")
                .default_value("none")
                .value_name("NAME")
                .help("An optional name of a master group for a sentinel Redis connection.")
        ).arg(
            Arg::with_name("redis-set")
                .short("s")
                .long("redis-set")
                .env("SCRAPER_REDIS_SET")
                .default_value("collection")
                .value_name("SET")
                .help("Specify the redis set to connect to")
        ).arg(
            Arg::with_name("log-path")
                .short("l")
                .long("log-path")
                .env("SCRAPER_WORKER_LOG_PATH")
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
        ).arg(
            Arg::with_name("metrics-enable")
                .short("d")
                .long("enable-metrics")
                .env("SCRAPER_METRICS_ENABLE")
                .default_value("false")
                .value_name("BOOLEAN")
                .help("Specify whether to enable metric logging")
        ).arg(
            Arg::with_name("influx-addr")
                .short("g")
                .long("influx-addr")
                .env("SCRAPER_METRICS_INFLUXDB_ADDR")
                .default_value("localhost")
                .value_name("STRING")
                .help("Specify InfluxDB address")
        ).arg(
            Arg::with_name("influx-port")
                .short("h")
                .long("influx-port")
                .env("SCRAPER_METRICS_INFLUXDB_PORT")
                .default_value("8086")
                .value_name("INT")
                .help("Specify InfluxDB port")
        ).arg(
            Arg::with_name("influx-username")
                .short("i")
                .long("influx-user")
                .env("SCRAPER_METRICS_INFLUXDB_USER")
                .default_value("worker")
                .value_name("STRING")
                .help("Specify InfluxDB username")
        ).arg(
            Arg::with_name("influx-password")
                .short("j")
                .long("influx-password")
                .env("SCRAPER_METRICS_INFLUXDB_PASSWORD")
                .default_value("password")
                .value_name("STRING")
                .help("Specify InfluxDB password")
        ).arg(
            Arg::with_name("influx-database")
                .short("k")
                .long("influx-database")
                .env("SCRAPER_METRICS_INFLUXDB_DATABASE")
                .default_value("scraper_db")
                .value_name("STRING")
                .help("Specify InfluxDB database")
        ).get_matches();

    // Load config for logging to stdout and logfile.
    if let Ok(_handle) = log4rs::init_config(get_log4rs_config(
        args.value_of("log-path").unwrap(),
        match args.value_of("log-level").unwrap().to_lowercase().as_str() {
            "error" => LevelFilter::Error,
            "warn" => LevelFilter::Warn,
            "info" => LevelFilter::Info,
            "debug" => LevelFilter::Debug,
            "trace" => LevelFilter::Trace,
            "off" => LevelFilter::Off,
            _ => LevelFilter::Off,
        },
    )) {
        info!("Build commit: {}", env!("VERGEN_SHA"));

        info!(
            "Starting worker module using RabbitMQ({}:{}) and redis({}:{})",
            args.value_of("rmq-address").unwrap().to_string(),
            args.value_of("rabbitmq-port").unwrap().to_string(),
            args.value_of("redis-address").unwrap().to_string(),
            args.value_of("redis-port").unwrap().to_string(),
        );

        let sentinel_arg: &str = args.value_of("sentinel").unwrap();
        let sentinel = if sentinel_arg != "none" {
            Some(sentinel_arg)
        } else {
            None
        };

        // Construct a worker and its components
        let manager = RMQRedisManager::new(
            args.value_of("rmq-address").unwrap().to_string(),
            args.value_of("rabbitmq-port").unwrap().parse().expect("Failed parsing Rabbitmq port to u16"), // Parse str to u16
            args.value_of("redis-address").unwrap().to_string(),
            args.value_of("redis-port").unwrap().parse().expect("Failed parsing Redis port to u16"), // Parse str to u16
            args.value_of("rabbitmq-exchange").unwrap().to_string(),
            args.value_of("rabbitmq-prefetch-count").unwrap().parse().expect("Failed parsing prefetch count to u16"), // Parse str to u16
            args.value_of("rabbitmq-queue").unwrap().to_string(),
            args.value_of("rabbitmq-collection-queue").unwrap().to_string(),
            args.value_of("redis-set").unwrap().to_string(),
            sentinel,
        ).expect("Failed to construct RMQRedisManager");
        let downloader = DefaultDownloader::new();
        let extractor = HTMLExtractorBase::new(HTMLLinkExtractor::new());
        let filter: Box<dyn Filter> = if args.value_of("filter-enable").unwrap().parse().unwrap() {
            match args.value_of("filter-type").unwrap() {
                "black" => Box::new(Blacklist::new(
                    args.value_of("filter-path").unwrap().to_string(),
                )),
                "white" | _ => Box::new(Whitelist::new(
                    args.value_of("filter-path").unwrap().to_string(),
                )),
            }
        } else {
            Box::new(NoFilter)
        };
        let normaliser = DefaultNormaliser;
        let archive = Void;
        let worker = Worker::new(
            "W1",
            Box::new(manager),
            Box::new(downloader),
            Box::new(extractor),
            Box::new(normaliser),
            Box::new(archive),
            filter,
        );

        let influxdb_client = if args
            .value_of("metrics-enable")
            .unwrap()
            .parse()
            .expect("The 'metrics-enable' argument was not a boolean")
        {
            Some(InfluxClient::new(
                args.value_of("influx-addr").unwrap(),
                args.value_of("influx-port")
                    .unwrap()
                    .parse()
                    .expect("The 'influx-port' argument was not an int"),
                args.value_of("influx-username").unwrap(),
                args.value_of("influx-password").unwrap(),
                args.value_of("influx-database").unwrap(),
            ))
        } else {
            None
        };

        worker.start(influxdb_client);

        Ok(())
    } else {
        Err(Box::new(std::io::Error::new(
            ErrorKind::Other,
            "[ERROR] Failed creating logging config",
        )))
    }
}
