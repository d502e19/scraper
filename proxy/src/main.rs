extern crate redis;
extern crate futures;
extern crate lapin_futures;
extern crate tokio;

mod task;

use std::error::Error;
use redis::Commands;
use redis::RedisResult;
use futures::future::Future;
use futures::stream::Stream;
use lapin_futures::options::{
    BasicConsumeOptions, BasicRejectOptions, QueueDeclareOptions, ExchangeDeclareOptions, QueueBindOptions,
};
use lapin_futures::types::FieldTable;
use lapin_futures::{Client, ConnectionProperties, ExchangeKind};
use crate::task::Task;


fn main() -> Result<(), Box<dyn Error>> {
    // tries to get a connection to redis
    // if a connection is established continue else there is an error
    let client = redis::Client::open("redis://192.168.99.100:6379/").unwrap();
    let con = client.get_connection();
    match con {
        Ok(mut connection) => {
            // establish a connection to rabbitMQ
            let addr = std::env::var("AMQP_ADDR").unwrap_or_else(|_| "amqp://192.168.99.100:5672/%2f".into());
            futures::executor::spawn(
                Client::connect(&addr, ConnectionProperties::default()).and_then(|client| {
                    // finds collection and sees the tasks
                    client.create_channel().and_then(|channel| {
                        channel.exchange_declare("work", ExchangeKind::Fanout,
                                                 ExchangeDeclareOptions::default(), FieldTable::default());
                        channel.queue_declare("collection", QueueDeclareOptions::default(),
                                              FieldTable::default(), ).and_then(move |queue| {
                            channel.queue_bind("collection", "work", "",
                                               QueueBindOptions::default(), FieldTable::default());
                            channel.basic_consume(&queue, "", BasicConsumeOptions::default(),
                                                  FieldTable::default(), ).and_then(|consumer| {
                                // copies every task from collection to redis
                                consumer.for_each(move |msg| {
                                    let task = Task::deserialise(msg.data).unwrap();
                                    let add_res: RedisResult<u32> = connection.sadd("collection", &task);
                                    if let Ok(s) = add_res {
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
                })
            ).wait_future().expect("Could not connect to rabbitMQ");
        }
        Err(_) =>
            println!("Could not connect to redis"),
    }
    Ok(())
}