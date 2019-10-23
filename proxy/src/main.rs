extern crate redis;
extern crate futures;
extern crate lapin_futures;
extern crate tokio;

mod task;

use std::error::Error;
use redis::Commands;
use redis::RedisResult;
use std::collections::HashSet;
use futures::future::Future;
use futures::stream::Stream;
use lapin_futures::options::{
    BasicConsumeOptions, BasicPublishOptions, BasicRejectOptions, QueueDeclareOptions, ExchangeDeclareOptions, QueueBindOptions
};
use lapin_futures::types::FieldTable;
use lapin_futures::{BasicProperties, Channel, Client, ConnectionProperties, Queue, ExchangeKind};
use futures::IntoFuture;
use std::io::BufRead;
use std::sync::mpsc::channel;
use std::str::from_utf8;
use std::str;
use std::string::String;
use crate::task::Task;


fn main() -> Result<(), Box<dyn Error>> {
    let client = redis::Client::open("redis://192.168.99.100:6379/").unwrap();
    let mut con = client.get_connection();
    let exptask = Task { url :  "aau.dk".to_string()};
    match con {
        Ok(mut connection ) => {
            let addr = std::env::var("AMQP_ADDR").unwrap_or_else(|_| "amqp://192.168.99.100:5672/%2f".into());

            futures::executor::spawn(
            Client::connect(&addr, ConnectionProperties::default()).and_then(|client| {
                println!("CONNECTED");
              client.create_channel().and_then(|channel| {
                  channel.exchange_declare("work", ExchangeKind::Fanout,
                                           ExchangeDeclareOptions::default(), FieldTable::default());
                  channel.queue_declare("collection", QueueDeclareOptions::default(),
                                        FieldTable::default(),).and_then(move |queue |{
                      channel.queue_bind("collection", "work", "",
                                         QueueBindOptions::default(), FieldTable::default());
                      channel.basic_publish("work", "", exptask.serialise(),  BasicPublishOptions::default(),
                                            BasicProperties::default(),).wait();

                      channel.basic_consume(&queue, "", BasicConsumeOptions::default(),
                                                                 FieldTable::default(),).and_then(|consumer|{
                              consumer.for_each(move |msg| {
                                  let task = Task::deserialise(msg.data);
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

            ).wait_future().expect("error");
        }
        Err(_) =>
            println!("error"),
    }

    Ok(())
}