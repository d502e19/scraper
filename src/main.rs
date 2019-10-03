extern crate futures;
extern crate tokio;
extern crate lapin_futures;

mod frontier;
mod task;

use futures::future::Future;
use lapin_futures as lapin;
use crate::lapin::ExchangeKind;
use crate::lapin::{BasicProperties, Client, ConnectionProperties};
use crate::lapin::options::{BasicPublishOptions, QueueDeclareOptions, ExchangeDeclareOptions, QueueBindOptions, BasicConsumeOptions, QueuePurgeOptions};
use crate::lapin::types::FieldTable;
use tokio::prelude::Stream;
use lapin_futures::Channel;
use crate::task::Task;

fn main() {
    let addr = "amqp://192.168.99.100:5672/%2f";

    let msg = Task { url: String::from("https://aau.dk") };

    futures::executor::spawn(
        Client::connect(&addr, ConnectionProperties::default()).and_then(|client| {
            // create_channel returns a future that is resolved
            // once the channel is successfully created
            client.create_channel()
        }).and_then(|channel| {
            let id = channel.id();
            println!("created channel with id: {}", id);

            // we using a "move" closure to reuse the channel
            // once the queue is declared. We could also clone
            // the channel
            channel.queue_declare("frontier",
                                  QueueDeclareOptions::default(),
                                  FieldTable::default()).and_then(move |queue| {
                println!("channel {} declared queue 'frontier'", id);
                channel.queue_purge("frontier", QueuePurgeOptions::default());

                channel.exchange_declare("work",
                                         ExchangeKind::Fanout,
                                         ExchangeDeclareOptions::default(),
                                         FieldTable::default()).and_then(move |_| {
                    println!("channel {} declared exchange 'work'", id);

                    channel.queue_bind("frontier",
                                       "work",
                                       "",
                                       QueueBindOptions::default(),
                                       FieldTable::default()).and_then(move |_| {
                        println!("channel {} bound 'work' to 'frontier'", id);

                        channel.basic_publish("work", "", msg.serialise(),
                                              BasicPublishOptions::default(), BasicProperties::default()).wait().expect("publish");

                        channel.basic_consume(&queue,
                                              "my_consumer",
                                              BasicConsumeOptions::default(),
                                              FieldTable::default()).and_then(move |consumer| {
                            println!("channel {} created consumer", id);

                            // Runs until error occurs
                            consumer.for_each(move |dev| {
                                let task = Task::deserialise(dev.data);
                                println!("msg: {}", task.url);

                                let first_new_task = Task { url: format!("{}/0", task.url) };
                                let second_new_task = Task { url: format!("{}/1", task.url) };

                                channel.basic_publish("work", "", first_new_task.serialise(),
                                                      BasicPublishOptions::default(), BasicProperties::default()).wait().expect("publish");
                                channel.basic_publish("work", "", second_new_task.serialise(),
                                                      BasicPublishOptions::default(), BasicProperties::default()).wait().expect("publish");

                                channel.basic_ack(dev.delivery_tag, false)
                            })
                        })
                    })
                })
            })
        })
    ).wait_future().expect("runtime failure");
}
