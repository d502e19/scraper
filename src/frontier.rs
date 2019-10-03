use crate::task::Task;
use lapin_futures::{Channel, Client, ConnectionProperties, ExchangeKind, Queue, BasicProperties};
use lapin_futures::options::{BasicPublishOptions, QueueDeclareOptions, ExchangeDeclareOptions, QueueBindOptions, BasicConsumeOptions};
use lapin_futures::types::FieldTable;
use futures::Future;
use tokio::prelude::Stream;

pub trait Frontier {
    fn submit_task(&self, task: Task) -> Result<(), ()>;

    fn subscribe(&self, f: Box<dyn Fn(Task) -> TaskSubmitResult>);

    fn close(self) -> Result<(), ()>;
}

pub enum TaskSubmitResult {
    Ok,
    Err,
    Reject,
}

pub struct RabbitmqFrontier {
    channel: Channel,
    queue: Queue,
}

impl RabbitmqFrontier {
    pub(crate) fn new(addr: String) -> Result<Self, ()> {
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
                                  FieldTable::default()).and_then(move |queue| {
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

                        Ok(RabbitmqFrontier {
                            channel,
                            queue,
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

impl Frontier for RabbitmqFrontier {
    fn submit_task(&self, task: Task) -> Result<(), ()> {
        match self.channel.basic_publish("work", "", task.serialise(), BasicPublishOptions::default(), BasicProperties::default()).wait() {
            Ok(_) => Ok(()),
            Err(_) => Err(()),
        }
    }

    fn subscribe(&self, f: Box<dyn Fn(Task) -> TaskSubmitResult>) {
        self.channel.basic_consume(&self.queue, "my_consumer", BasicConsumeOptions::default(), FieldTable::default()).and_then(
            move |consumer| {
                consumer.for_each(move |dev| {
                    let task = Task::deserialise(dev.data);
                    let result = f(task);
                    match result {
                        TaskSubmitResult::Ok => {
                            self.channel.basic_ack(dev.delivery_tag, false)
                        }
                        TaskSubmitResult::Reject => {
                            // No op, so the task is rescheduled
                            unimplemented!("Ended in Reject-state")
                        }
                        TaskSubmitResult::Err => {
                            panic!("Processing task failed")
                        }
                    }
                })
            }
        ).wait().expect("Subscription failed");
    }

    fn close(self) -> Result<(), ()> {
        match self.channel.close(0u16, "Client closed connection").wait() {
            Ok(_) => Ok(()),
            Err(_) => Err(())
        }
    }
}
