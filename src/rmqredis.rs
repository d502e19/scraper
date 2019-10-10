use crate::traits::{FrontierSubmitted, TaskProcessResult};
use crate::task::Task;
use lapin_futures::{Channel, BasicProperties, Queue, Client, ConnectionProperties};
use lapin_futures::options::{
    BasicPublishOptions,
    BasicConsumeOptions,
    BasicRejectOptions,
    QueueDeclareOptions
};
use futures::future::Future;
use lapin_futures::types::FieldTable;
use futures::stream::Stream;

pub struct RMQRedis {
    channel: Channel,
    queue: Queue,
    exchange: String,
    routing_key: String,
}

impl RMQRedis {
    fn new(addr: &str, exchange: String, routing_key: String, queue: &str) -> Result<RMQRedis, lapin_futures::Error> {
        Client::connect(addr, ConnectionProperties::default())
            .and_then(|client| {
                client.create_channel().and_then(|channel| {
                    channel.queue_declare(
                        queue,
                        QueueDeclareOptions::default(),
                        FieldTable::default(),
                    ).and_then(|queue| {
                        Ok(RMQRedis {
                            channel,
                            queue,
                            exchange,
                            routing_key
                        })
                    })
                })
            })
            .wait()
    }
}

impl FrontierSubmitted for RMQRedis {
    fn submit_task(&self, task: &Task) -> Result<(), ()> {
        let result = self.channel.basic_publish(
            self.exchange.as_str(),
            self.routing_key.as_str(),
            task.serialise(),
            BasicPublishOptions::default(),
            BasicProperties::default(),
        ).wait();

        match result {
            Ok(_) => Ok(()),
            Err(_) => Err(()),
        }
    }

    fn start_listening<F>(&self, f: F) where
        F: Fn(&Task) -> TaskProcessResult {
        self.channel.basic_consume(
            &self.queue,
            "", //TODO
            BasicConsumeOptions::default(),
            FieldTable::default()
        ).and_then(move |consumer| {
            consumer.for_each(move |delivery| {
                let task = Task::deserialise(delivery.data);
                let result = f(&task);
                match result {
                    TaskProcessResult::Ok => {
                        //TODO submit result to data storage
                        unimplemented!()
                    },
                    TaskProcessResult::Err => {
                        self.channel.basic_reject(
                            delivery.delivery_tag,
                            BasicRejectOptions::default(), //TODO
                        );
                    },
                    TaskProcessResult::Reject => {
                        self.channel.basic_reject(
                            delivery.delivery_tag,
                            BasicRejectOptions::default(), //TODO
                        );
                    },
                }

                Ok(())
            })
        }).wait().unwrap();
    }

    fn close(self) -> Result<(), ()> {
        self.channel.close(0, "called close()");
        unimplemented!()
    }

    fn contains(&self, task: &Task) -> Result<bool, ()> {
        // TODO Query Redis directly
        // Discussion: Should we query Redis through RabbitMQ pub-sub into to decouple in space and time?
        unimplemented!()
    }
}