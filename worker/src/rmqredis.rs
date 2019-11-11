use std::io::{Error, ErrorKind};

use futures::future::Future;
use futures::stream::Stream;
use lapin_futures::{BasicProperties, Channel, Client, ConnectionProperties, ExchangeKind, Queue};
use lapin_futures::options::{
    BasicConsumeOptions, BasicPublishOptions, BasicRejectOptions, ExchangeDeclareOptions,
    QueueBindOptions, QueueDeclareOptions, BasicQosOptions,
};
use lapin_futures::types::FieldTable;
use redis::{Commands, FromRedisValue, RedisError, RedisWrite, ToRedisArgs, Value};

use crate::errors::{ManagerError, ManagerResult};
use crate::errors::ManagerErrorKind::UnreachableError;
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
            Value::Data(ref bytes) => {
                Task::deserialise(bytes.to_owned())
                    .map_err(|_| RedisError::from(Error::new(ErrorKind::Other, "Failed to deserialise task")))
            }
            _ => Err(RedisError::from(Error::new(
                ErrorKind::Other,
                "Response could not be translated to a task",
            )))
        }
    }
}

pub struct RMQRedisManager {
    rmq_addr: String,
    rmq_port: u16,
    redis_addr: String,
    redis_port: u16,
    channel: Channel,
    queue: Queue,
    exchange: String,
    prefetch_count: u16,
    routing_key: String,
    redis_set: String,
}

impl RMQRedisManager {
    pub fn new(
        rmq_addr: String,
        rmq_port: u16,
        redis_addr: String,
        redis_port: u16,
        exchange: String,
        prefetch_count: u16,
        routing_key: String,
        queue_name: String,
        redis_set: String,
    ) -> Result<RMQRedisManager, ()> {
        debug!("Creating RMQRedisManager with following values: \n\trmq_addr: {:?}\n\trmq_port: {:?}\
            \n\t redis_addr: {:?}\n\tredis_port: {:?}\n\trmq_exchange: {:?}\n\tprefetch_count: {:?}\n\trmq_routing_key: {:?}\
            \n\trmq_queue_name: {:?}\n\tredis_set: {:?}"
               , rmq_addr, rmq_port, redis_addr, redis_port, exchange, prefetch_count, routing_key, queue_name, redis_set);

        let client = Client::connect(
            format!("amqp://{}:{}/%2f", rmq_addr, rmq_port).as_str(),
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

        // Limit the amount of tasks stored in the local queue
        channel.basic_qos(
            prefetch_count,
            BasicQosOptions::default(),
        ).wait().map_err(|_| ())?;

        Ok(RMQRedisManager {
            rmq_addr,
            rmq_port,
            redis_addr,
            redis_port,
            channel,
            queue,
            exchange,
            prefetch_count,
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

        result.map_err(|e| ManagerError::new(UnreachableError, "Could not reach manager.", Some(Box::new(e))))
    }

    fn start_listening(&self, resolve_func: &dyn Fn(Task) -> TaskProcessResult) {
        self.channel
            .basic_consume(
                &self.queue,
                "", //TODO
                BasicConsumeOptions::default(),
                FieldTable::default(),
            )
            .and_then(move |consumer| {
                consumer.for_each(move |delivery| {
                    let task_res = Task::deserialise(delivery.data);
                    match task_res {
                        Err(_) => {
                            self.channel.basic_reject(delivery.delivery_tag, BasicRejectOptions { requeue: false })
                        }
                        Ok(task) => {
                            let result = resolve_func(task);
                            match result {
                                TaskProcessResult::Ok => {
                                    self.channel.basic_ack(delivery.delivery_tag, false)
                                }
                                TaskProcessResult::Err => self
                                    .channel
                                    // Do not requeue task if error is met
                                    .basic_reject(delivery.delivery_tag, BasicRejectOptions { requeue: false }),
                                TaskProcessResult::Reject => self
                                    .channel
                                    // Do requeue task if error is met
                                    .basic_reject(delivery.delivery_tag, BasicRejectOptions { requeue: true }),
                            }
                        }
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
            redis::Client::open(format!("redis://{}:{}/", self.redis_addr, self.redis_port).as_str());
        if let Ok(client) = client_result {
            if let Ok(mut con) = client.get_connection() {
                let found_result = con.sismember(self.redis_set.as_str(), task);
                if let Ok(found) = found_result {
                    return Ok(found);
                }
            }
        }
        Err(ManagerError::new(UnreachableError, "Could not reach manager.", None))
    }
}
