extern crate clap;
extern crate futures;
extern crate lapin_futures;
#[macro_use]
extern crate log;
extern crate redis;
extern crate tokio;

use std::error::Error;
use std::io::ErrorKind;

use clap::{App, Arg};
use futures::future::Future;
use futures::stream::Stream;
use lapin_futures::{Client, ConnectionProperties};
use lapin_futures::options::{BasicConsumeOptions, BasicRejectOptions, QueueDeclareOptions};
use lapin_futures::types::FieldTable;
use log4rs::append::console::ConsoleAppender;
use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Config, Root};
use log4rs::encode::pattern::PatternEncoder;
use log::LevelFilter;
use redis::{Commands, RedisResult, IntoConnectionInfo, RedisError, Connection, ConnectionAddr, ConnectionInfo};

use crate::task::Task;

mod task;

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
    let args = App::new("DatScraper Proxy")
        .version("0.1.0")
        .author("d502e19@aau")
        .arg(
            Arg::with_name("redis-address")
                .short("e")
                .long("addr")
                .env("SCRAPER_REDIS_ADDRESS")
                // Checks for system at compile-time, not runtime
                .default_value(if cfg!(windows) {
                    "192.168.99.100"
                } else {
                    "localhost"
                })
                .value_name("ADDR")
                .help("Specify the redis address"),
        )
        .arg(
            Arg::with_name("redis-port")
                .short("r")
                .long("redis-port")
                .env("SCRAPER_REDIS_PORT")
                .default_value("6379")
                .value_name("PORT")
                .help("Specify the redis-port to connect to"),
        ).arg(
            Arg::with_name("sentinel")
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
            .help("Specify the redis set to connect to"),
        ).
        arg(
            Arg::with_name("rabbitmq-routing-key")
                .short("k")
                .long("rmq-routing-key")
                .env("SCRAPER_RABBITMQ_ROUTING_KEY")
                .default_value("") // No routing-key by default
                .value_name("KEY")
                .help("Specify the RabbitMQ routing-key to connect to")
        )
        .arg(
            Arg::with_name("rabbitmq-collection-queue")
                .short("d")
                .long("rmq-redis-queue")
                .env("SCRAPER_RABBITMQ_REDIS_QUEUE")
                .default_value("collection")
                .value_name("QUEUE")
                .help("Specify the RabbitMQ-REDIS queue to connect to"),
        )
        .arg(
            Arg::with_name("rabbitmq-consumer-tag")
                .short("t")
                .long("rmq-consumer-tag")
                .env("SCRAPER_RABBITMQ_CONSUMER_TAG")
                .default_value("proxy")
                .value_name("TAG")
                .help("Specify the RabbitMQ consumer tag to use"),
        )
        .arg(
            Arg::with_name("rmq-port")
                .short("p")
                .long("rmq-port")
                .env("SCRAPER_RABBITMQ_PORT")
                .default_value("5672")
                .value_name("PORT")
                .help("Specify the RabbitMQ port to connect to"),
        )
        .arg(
            Arg::with_name("rmq-address")
                .short("a")
                .long("rmq-addr")
                .env("SCRAPER_RMQ_ADDRESS")
                // Checks for system at compile-time, not runtime
                .default_value(if cfg!(windows) {
                    "192.168.99.100"
                } else {
                    "localhost"
                })
                .value_name("ADDR")
                .help("Specify the RabbitMQ address"),
        )
        .arg(
            Arg::with_name("log-path")
                .short("l")
                .long("log-path")
                .env("SCRAPER_PROXY_LOG_PATH")
                .default_value("proxy.log")
                .value_name("PATH")
                .help("Specify the log-file path")
        )
        .arg(
            Arg::with_name("log-level")
                .short("o")
                .long("log-level")
                .env("LOG_LEVEL")
                .default_value("info")
                .value_name("LEVEL")
                .help("Specify the log level {error, warn, info, debug, trace, off}")
        ).get_matches();

    // Load config for logging to stdout and logfile.
    if let Ok(_handle) = log4rs::init_config(
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
        info!("Build commit: {}", env!("VERGEN_SHA"));

        // Establish Redis connection
        let sentinel_arg: &str = args.value_of("sentinel").unwrap();
        let sentinel = if sentinel_arg != "none" {
            Some(sentinel_arg)
        } else {
            None
        };
        let connection_info = format!(
            "redis://{}:{}/",
            args.value_of("redis-address").unwrap(),
            args.value_of("redis-port").unwrap(),
        ).as_str().into_connection_info()?;
        let mut connection = create_redis_connection(connection_info, sentinel)?;

        // Establish a connection to RabbitMQ using env-var or passed arg
        let rmq_addr = format!(
            "amqp://{}:{}/%2f",
            args.value_of("rmq-address").unwrap(),
            args.value_of("rmq-port").unwrap(),
        );
        let client = Client::connect(&rmq_addr, ConnectionProperties::default()).wait()?;
        // Declare the collection queue
        let channel = client.create_channel().wait()?;
        let queue = channel
            .queue_declare(
                args.value_of("rabbitmq-collection-queue").unwrap(),
                QueueDeclareOptions::default(),
                FieldTable::default(),
            ).wait()?;

        let consumer = channel
            .basic_consume(
                &queue,
                args.value_of("rabbitmq-consumer-tag").unwrap(),
                BasicConsumeOptions::default(),
                FieldTable::default(),
            ).wait()?;

        info!("Proxy has started!");

        // Copies every task from collection to redis
        let redis_set = args.value_of("redis-set").unwrap();
        consumer.for_each(move |msg| {
            let received_task =
                Task::deserialise(msg.data).unwrap();
            let add_res: RedisResult<u32> = connection.sadd(
                redis_set,
                &received_task,
            );
            if let Ok(_) = add_res {
                channel.basic_ack(msg.delivery_tag, false)
            } else {
                channel.basic_reject(
                    msg.delivery_tag,
                    BasicRejectOptions { requeue: true },
                )
            }
        }).wait()?;

        Ok(())
    } else {
        Err(Box::new(std::io::Error::new(ErrorKind::Other, "[ERROR] Failed creating logging config")))
    }
}

/// Establishes a redis connection. An optional name of a master group can be given to
/// make the connection sentinel.
fn create_redis_connection(connection_info: ConnectionInfo, sentinel: Option<&str>) -> Result<Connection, RedisError> {
    let mut client = redis::Client::open(connection_info.clone())?;

    if let Some(name) = sentinel {
        // Get details about the Redis master
        let (master_addr, master_port) = redis::cmd("SENTINEL")
            .arg("get-master-addr-by-name")
            .arg(name)
            .query::<(String, u16)>(&mut client)?;

        // New sentinel client using master address and master port
        let sentinel_client = redis::Client::open(
            ConnectionInfo {
                addr: Box::new(ConnectionAddr::Tcp(master_addr, master_port)),
                ..connection_info
            },
        )?;

        return sentinel_client.get_connection()

    } else {
        // Non-sentinel connection
        return client.get_connection()
    }
}