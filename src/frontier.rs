use crate::task::Task;
use lapin_futures::{Channel, Client, ConnectionProperties, ExchangeKind};
use lapin_futures::options::{BasicPublishOptions, QueueDeclareOptions, ExchangeDeclareOptions, QueueBindOptions, BasicConsumeOptions, QueuePurgeOptions};
use lapin_futures::types::FieldTable;
use futures::Future;

pub trait Queue {
    fn submit_task(&self, task: Task) -> Result<(), ()>;

    fn subscribe(&self, f: Box<dyn Fn(Task) -> TaskSubmitResult>);

    fn close(self) -> Result<(), ()>;
}

pub enum TaskSubmitResult {
    Ok,
    Err,
    Reject,
}

pub struct RabbitmqQueue {
    channel: Channel,
}

impl RabbitmqQueue {
    fn new(addr: String) -> Result<Self, ()> {
        match Client::connect(&addr, ConnectionProperties::default()).and_then(|client| {
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
                                  FieldTable::default()).and_then( move |queue| {
                println!("channel {} declared queue 'frontier'", id);

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

                        Ok(RabbitmqQueue {
                            channel
                        })
                    })
                })
            })
        }).wait() {
            Ok(q) => Ok(q),
            Err(_) => Err(()),
        }
    }
}

impl Queue for RabbitmqQueue {
    fn submit_task(&self, task: Task) -> Result<(), ()> {
        unimplemented!()
    }

    fn subscribe(&self, f: Box<dyn Fn(Task) -> TaskSubmitResult>) {
        unimplemented!()
    }

    fn close(self) -> Result<(), ()> {
        unimplemented!()
    }
}