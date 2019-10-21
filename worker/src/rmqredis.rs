use crate::task::Task;
use crate::traits::{Manager, TaskProcessResult};
use futures::future::Future;
use futures::stream::Stream;
use lapin_futures::options::{
    BasicConsumeOptions, BasicPublishOptions, BasicRejectOptions, QueueDeclareOptions,
};
use lapin_futures::types::FieldTable;
use lapin_futures::{BasicProperties, Channel, Client, ConnectionProperties, Queue};
use redis::{Commands, FromRedisValue, RedisError, RedisResult, RedisWrite, ToRedisArgs, Value};
use std::str::from_utf8;
use url::Url;
use std::io::{Error, ErrorKind};

// Allows Redis to automatically serialise Task into raw bytes with type inference
impl ToRedisArgs for &Task {
    fn write_redis_args<W>(&self, out: &mut W)
    where
        W: ?Sized + RedisWrite,
    {
        out.write_arg(self.url.as_str().as_bytes())
    }
}

// Allows Redis to automatically deserialise Task from raw bytes with type inference
impl FromRedisValue for Task {
    fn from_redis_value(v: &Value) -> Result<Self, RedisError> {
        match *v {
            Value::Data(ref bytes) => Ok(Task::deserialise(bytes.to_owned())),
            _ => Err(RedisError::from(Error::new(ErrorKind::Other, "Response could not be translated to a task"))),
        }
    }
}

pub struct RMQRedisManager {
    addr: String,
    rmq_port: String,
    redis_port: String,
    channel: Channel,
    queue: Queue,
    exchange: String,
    routing_key: String,
    redis_set: String,
}

impl RMQRedisManager {
    fn new(
        addr: String,
        rmq_port: String,
        redis_port: String,
        exchange: String,
        routing_key: String,
        queue: String,
        redis_set: String,
    ) -> Result<RMQRedisManager, ()> {
        Client::connect(
            format!("amqp://{}:{}/%2f", addr, rmq_port).as_str(),
            ConnectionProperties::default(),
        )
        .and_then(|client| {
            client.create_channel().and_then(|channel| {
                channel
                    .queue_declare(
                        queue.as_str(),
                        QueueDeclareOptions::default(),
                        FieldTable::default(),
                    )
                    .and_then(|queue| {
                        Ok(RMQRedisManager {
                            addr,
                            rmq_port,
                            redis_port,
                            channel,
                            queue,
                            exchange,
                            routing_key,
                            redis_set,
                        })
                    })
            })
        })
        .wait()
        .map_err(|_| ())
    }
}

impl Manager for RMQRedisManager {
    fn submit_task(&self, task: &Task) -> Result<(), ()> {
        let result = self
            .channel
            .basic_publish(
                self.exchange.as_str(),
                self.routing_key.as_str(),
                task.serialise(),
                BasicPublishOptions::default(),
                BasicProperties::default(),
            )
            .wait();

        match result {
            Ok(_) => Ok(()),
            Err(_) => Err(()),
        }
    }

    fn start_listening<F>(&self, f: F)
    where
        F: Fn(&Task) -> TaskProcessResult,
    {
        self.channel
            .basic_consume(
                &self.queue,
                "", //TODO
                BasicConsumeOptions::default(),
                FieldTable::default(),
            )
            .and_then(move |consumer| {
                consumer.for_each(move |delivery| {
                    let task = Task::deserialise(delivery.data);
                    let result = f(&task);
                    match result {
                        TaskProcessResult::Ok => {
                            self.channel.basic_ack(delivery.delivery_tag, false)
                        }
                        TaskProcessResult::Err => {
                            self.channel.basic_reject(
                                delivery.delivery_tag,
                                BasicRejectOptions::default(),
                            )
                        }
                        TaskProcessResult::Reject => {
                            self.channel.basic_reject(
                                delivery.delivery_tag,
                                BasicRejectOptions::default(),
                            )
                        }
                    }
                })
            })
            .wait()
            .unwrap();
    }

    fn close(self) -> Result<(), ()> {
        self.channel.close(0, "called close()");
        Ok(())
    }

    fn contains(&self, task: &Task) -> Result<bool, ()> {
        let client_result =
            redis::Client::open(format!("redis://{}:{}/", self.addr, self.redis_port).as_str());
        if let Ok(client) = client_result {
            if let Ok(mut con) = client.get_connection() {
                let found_result = con.sismember(self.redis_set.as_str(), task);
                if let Ok(found) = found_result {
                    return Ok(found);
                }
            }
        }
        Err(())
    }
}
