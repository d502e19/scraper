extern crate clap;
extern crate futures;
extern crate lapin_futures;
extern crate redis;
extern crate tokio;

use std::error::Error;

use clap::{App, Arg};
use futures::future::Future;
use futures::stream::Stream;
use lapin_futures::options::{BasicConsumeOptions, BasicRejectOptions, QueueDeclareOptions};
use lapin_futures::types::FieldTable;
use lapin_futures::{Client, ConnectionProperties};
use redis::{Commands, RedisResult};

use crate::task::Task;

mod task;

fn main() -> Result<(), Box<dyn Error>> {
    // Set up arguments and get resulting arguments
    let args = App::new("DatScraper Proxy")
        .version("0.1.0")
        .author("d502e19@aau")
        .arg(
            Arg::with_name("redis-address")
                .short("a")
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
        )
        .arg(
            Arg::with_name("redis-set")
                .short("s")
                .long("redis-set")
                .env("SCRAPER_REDIS_SET")
                .default_value("collection")
                .value_name("SET")
                .help("Specify the redis set to connect to"),
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
        .get_matches();

    // Tries to get a connection to redis
    // If a connection is established continue handling messages, otherwise put error
    let client = redis::Client::open(
        format!(
            "redis://{}:{}/",
            args.value_of("redis-address").unwrap(),
            args.value_of("redis-port").unwrap()
        )
        .as_str(),
    )
    .unwrap();
    let con = client.get_connection();
    match con {
        Ok(mut connection) => {
            // Establish a connection to rabbitMQ using env-var or passed arg FIXME; is this right?
            let addr = std::env::var("AMQP_ADDR").unwrap_or_else(|_| {
                format!(
                    "amqp://{}:{}/%2f",
                    args.value_of("redis-address").unwrap(),
                    args.value_of("redis-port").unwrap()
                )
                .as_str()
                .into()
            });
            futures::executor::spawn(
                Client::connect(&addr, ConnectionProperties::default()).and_then(|client| {
                    // Finds collection and sees the tasks
                    client.create_channel().and_then(|channel| {
                        channel
                            .queue_declare(
                                args.value_of("rabbitmq-collection-queue").unwrap(),
                                QueueDeclareOptions::default(),
                                FieldTable::default(),
                            )
                            .and_then(move |queue| {
                                channel
                                    .basic_consume(
                                        &queue,
                                        args.value_of("rabbitmq-consumer-tag").unwrap(),
                                        BasicConsumeOptions::default(),
                                        FieldTable::default(),
                                    )
                                    .and_then(|consumer| {
                                        // Copies every task from collection to redis
                                        consumer.for_each(move |msg| {
                                            let received_task = Task::deserialise(msg.data);
                                            let add_res: RedisResult<u32> = connection.sadd(
                                                args.value_of("redis-set").unwrap(),
                                                &received_task,
                                            );
                                            if let Ok(_s) = add_res {
                                                channel.basic_ack(msg.delivery_tag, false)
                                            } else {
                                                channel.basic_reject(
                                                    msg.delivery_tag,
                                                    BasicRejectOptions::default(), //TODO
                                                )
                                            }
                                        })
                                    })
                            })
                    })
                }),
            )
            .wait_future()
            .expect("Could not connect to rabbitMQ");
        }
        Err(_) => eprintln!("Could not connect to redis"),
    }
    Ok(())
}
