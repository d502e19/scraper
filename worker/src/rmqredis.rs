use std::io::{Error, ErrorKind};
use std::str::from_utf8;

use futures::future::Future;
use futures::stream::Stream;
use lapin_futures::{BasicProperties, Channel, Client, ConnectionProperties, ExchangeKind, Queue};
use lapin_futures::options::{
    BasicConsumeOptions, BasicPublishOptions, BasicRejectOptions, ExchangeDeclareOptions,
    QueueBindOptions, QueueDeclareOptions,
};
use lapin_futures::types::FieldTable;
use redis::{Commands, FromRedisValue, RedisError, RedisResult, RedisWrite, ToRedisArgs, Value};
use url::Url;

use crate::errors::{ManagerError, ManagerErrorKind, ManagerResult};
use crate::task::Task;
use crate::traits::{Manager, TaskProcessResult};

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
            _ => Err(RedisError::from(Error::new(
                ErrorKind::Other,
                "Response could not be translated to a task",
            ))),
        }
    }
}

pub struct RMQRedisManager {
    addr: String,
    rmq_port: u16,
    redis_port: u16,
    channel: Channel,
    queue: Queue,
    exchange: String,
    routing_key: String,
    redis_set: String,
}

impl RMQRedisManager {
    pub fn new(
        addr: String,
        rmq_port: u16,
        redis_port: u16,
        exchange: String,
        routing_key: String,
        queue_name: String,
        redis_set: String,
    ) -> Result<RMQRedisManager, ()> {
        // FIXME; use loglevel = info to allow this block
        {
            println!("addr: {:?}", addr);
            println!("rmq_port: {:?}", rmq_port);
            println!("redis_port: {:?}", redis_port);
            println!("rmq_exchange: {:?}", exchange);
            println!("rmq_routing_key: {:?}", routing_key);
            println!("rmq_queue_name: {:?}", queue_name);
            println!("redis_set: {:?}", redis_set);
        }
        let client = Client::connect(
            format!("amqp://{}:{}/%2f", addr, rmq_port).as_str(),
            ConnectionProperties::default(),
        ).wait().map_err(|_| ())?;

        let channel = client.create_channel().wait().map_err(|_| ())?;

        let queue = channel.queue_declare(
            queue_name.as_str(),
            QueueDeclareOptions::default(),
            FieldTable::default(),
        ).wait().map_err(|_| ())?;

        channel.exchange_declare(
            exchange.as_str(),
            ExchangeKind::Fanout,
            ExchangeDeclareOptions::default(),
            FieldTable::default(),
        ).wait().map_err(|_| ())?;

        channel.queue_bind(
            queue_name.as_str(),
            exchange.as_str(),
            routing_key.as_str(),
            QueueBindOptions::default(),
            FieldTable::default(),
        ).wait().map_err(|_| ())?;

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
    }
}

impl Manager for RMQRedisManager {
    fn submit_task(&self, task: &Task) -> ManagerResult<()> {
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

        result.map_err(|e| ManagerError::new(ManagerErrorKind::UnreachableError, String::from("Could not reach manager."), Some(Box::new(e))))
    }

    fn start_listening<F>(&self, f: F)
    where
        F: Fn(Task) -> TaskProcessResult,
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
                    let result = f(task);
                    match result {
                        TaskProcessResult::Ok => {
                            self.channel.basic_ack(delivery.delivery_tag, false)
                        }
                        TaskProcessResult::Err => self
                            .channel
                            .basic_reject(delivery.delivery_tag, BasicRejectOptions::default()),
                        TaskProcessResult::Reject => self
                            .channel
                            .basic_reject(delivery.delivery_tag, BasicRejectOptions::default()),
                    }
                })
            })
            .wait()
            .unwrap();
    }

    fn close(self) -> ManagerResult<()> {
        self.channel.close(0, "called close()");
        Ok(())
    }

    fn contains(&self, task: &Task) -> ManagerResult<bool> {
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
        Err(ManagerError::new(ManagerErrorKind::UnreachableError, String::from("Could not reach manager."), None))
    }
}
