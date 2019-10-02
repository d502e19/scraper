extern crate futures;
extern crate tokio;
extern crate lapin_futures;

use futures::future::Future;
use lapin_futures as lapin;
use crate::lapin::ExchangeKind;
use crate::lapin::{BasicProperties, Client, ConnectionProperties};
use crate::lapin::options::{BasicPublishOptions, QueueDeclareOptions, ExchangeDeclareOptions, QueueBindOptions};
use crate::lapin::types::FieldTable;

struct Task {
    url: String,
}

impl Task {
    fn serialise(self) -> Vec<u8> {
        self.url.into_bytes()
    }

    fn deserialise(data: Vec<u8>) -> Self {
        Task { url: String::from_utf8(data).unwrap() }
    }
}

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
                                  FieldTable::default()).and_then(move |_| {
                channel.exchange_declare("work",
                                         ExchangeKind::Fanout,
                                         ExchangeDeclareOptions::default(),
                                         FieldTable::default()).and_then(move |_| {
                    channel.queue_bind("frontier",
                                       "work",
                                       "",
                                       QueueBindOptions::default(),
                                       FieldTable::default());

                    println!("channel {} declared queue {}", id, "frontier");

                    channel.basic_publish("work", "", msg.serialise(),
                                          BasicPublishOptions::default(), BasicProperties::default())
                })
            })
        })
    ).wait_future().expect("runtime failure");
}
