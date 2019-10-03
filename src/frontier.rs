use crate::task::Task;
use lapin_futures::{Channel, Client, ConnectionProperties, ExchangeKind, Queue, BasicProperties};
use lapin_futures::options::{BasicPublishOptions, QueueDeclareOptions, ExchangeDeclareOptions, QueueBindOptions, BasicConsumeOptions};
use lapin_futures::types::FieldTable;
use futures::Future;
use tokio::prelude::Stream;

pub trait Frontier {
    fn submit_task(&self, task: Task) -> Result<(), ()>;

    fn start_listening<F>(&self, f: F) where F: Fn(Task) -> TaskProcessResult;

    fn close(self) -> Result<(), ()>;
}

pub enum TaskProcessResult {
    Ok,
    Err,
    Reject,
}

const RMQ_QUEUE: &str = "frontier";
const RMQ_EXCHANGE: &str = "work";
const RMQ_CONSUMER_NAME: &str = "my_consumer";

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
            channel.queue_declare(RMQ_QUEUE,
                                  QueueDeclareOptions::default(),
                                  FieldTable::default()).and_then(move |queue| {
                println!("channel {} declared queue '{}'", id, RMQ_QUEUE);

                channel.exchange_declare(RMQ_EXCHANGE,
                                         ExchangeKind::Fanout,
                                         ExchangeDeclareOptions::default(),
                                         FieldTable::default()).and_then(move |_| {
                    println!("channel {} declared exchange '{}'", id, RMQ_EXCHANGE);

                    channel.queue_bind(RMQ_QUEUE,
                                       RMQ_EXCHANGE,
                                       "",
                                       QueueBindOptions::default(),
                                       FieldTable::default()).and_then(move |_| {
                        println!("channel {} bound '{}' to '{}'", id, RMQ_EXCHANGE, RMQ_QUEUE);

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
        match self.channel.basic_publish(RMQ_EXCHANGE, "", task.serialise(), BasicPublishOptions::default(), BasicProperties::default()).wait() {
            Ok(_) => Ok(()),
            Err(_) => Err(()),
        }
    }

    fn start_listening<F>(&self, f: F) where F: Fn(Task) -> TaskProcessResult {
        self.channel.basic_consume(&self.queue, RMQ_CONSUMER_NAME, BasicConsumeOptions::default(), FieldTable::default()).and_then(
            move |consumer| {
                consumer.for_each(move |dev| {
                    let task = Task::deserialise(dev.data);
                    let result = f(task);
                    match result {
                        TaskProcessResult::Ok => {
                            self.channel.basic_ack(dev.delivery_tag, false).wait().expect("Failed to ack");
                            Ok(())
                        }
                        TaskProcessResult::Reject => {
                            // No op, so the task is rescheduled
                            unimplemented!("Ended in Reject-state")
                        }
                        TaskProcessResult::Err => {
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
