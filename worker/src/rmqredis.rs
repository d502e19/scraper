use std::io::{Error, ErrorKind};
use std::sync::{Mutex};

use futures::future::Future;
use futures::stream::Stream;
use lapin_futures::{BasicProperties, Channel, Client, ConnectionProperties, ExchangeKind, Queue};
use lapin_futures::options::{
    BasicConsumeOptions, BasicPublishOptions, BasicRejectOptions, ExchangeDeclareOptions,
    QueueBindOptions, QueueDeclareOptions, BasicQosOptions,
};
use lapin_futures::types::FieldTable;
use redis::{Commands, Connection, ConnectionAddr, FromRedisValue, RedisError, RedisWrite, ToRedisArgs, Value, ConnectionInfo, IntoConnectionInfo};

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

/// The RMQRedisManager is a Manager for a distributed web crawler that uses RabbitMQ and Redis.
/// Tasks are submitted to a RMQ exchange and received from a RMQ queue.
/// When checking if a task has already been submitted, the RQMRedisManager will ask Redis if
/// the task is in a given set.
pub struct RMQRedisManager {
    rmq_addr: String,
    rmq_port: u16,
    redis_addr: String,
    redis_port: u16,
    channel: Channel,
    frontier_queue: Queue,
    exchange: String,
    prefetch_count: u16,
    redis_connection: Mutex<Connection>,
    redis_set: String,
}

impl RMQRedisManager {
    /// Construct a new RMQRedisManager
    pub fn new(
        rmq_addr: String,
        rmq_port: u16,
        redis_addr: String,
        redis_port: u16,
        exchange: String,
        prefetch_count: u16,
        frontier_queue_name: String,
        collection_queue_name: String,
        redis_set: String,
    ) -> Result<RMQRedisManager, ()> {
        debug!("Creating RMQRedisManager with following values: \n\trmq_addr: {:?}\n\trmq_port: {:?}\
            \n\t redis_addr: {:?}\n\tredis_port: {:?}\n\trmq_exchange: {:?}\n\tprefetch_count: {:?}\
            \n\trmq_queue_name: {:?}\n\tcollection_queue_name: {:?}\n\tredis_set: {:?}"
               , rmq_addr, rmq_port, redis_addr, redis_port, exchange, prefetch_count, frontier_queue_name, collection_queue_name, redis_set);

        let client = Client::connect(
            format!("amqp://{}:{}/%2f", rmq_addr, rmq_port).as_str(),
            ConnectionProperties::default(),
        ).wait().map_err(|_| ())?;

        let channel = client.create_channel().wait().map_err(|_| ())?;

        let frontier_queue = channel.queue_declare(
            frontier_queue_name.as_str(),
            QueueDeclareOptions::default(),
            FieldTable::default(),
        ).wait().map_err(|_| ())?;

        channel.queue_declare(
            collection_queue_name.as_str(),
            QueueDeclareOptions::default(),
            FieldTable::default(),
        ).wait().map_err(|_|())?;

        channel.exchange_declare(
            exchange.as_str(),
            ExchangeKind::Fanout,
            ExchangeDeclareOptions::default(),
            FieldTable::default(),
        ).wait().map_err(|_| ())?;

        channel.queue_bind(
            frontier_queue_name.as_str(),
            exchange.as_str(),
            "",
            QueueBindOptions::default(),
            FieldTable::default(),
        ).wait().map_err(|_| ())?;

        channel.queue_bind(
            collection_queue_name.as_str(),
            exchange.as_str(),
            "",
            QueueBindOptions::default(),
            FieldTable::default(),
        ).wait().map_err(|_|())?;

        // Limit the amount of tasks stored in the local queue
        channel.basic_qos(
            prefetch_count,
            BasicQosOptions::default(),
        ).wait().map_err(|_| ())?;

        // Establish sentinel Redis connection
        let connection_info = format!("redis://{}:{}/", redis_addr, redis_port).as_str()
            .into_connection_info()
            .map_err(|_| ())?;
        let redis_connection = Mutex::new(
            create_sentinel_redis_connection(connection_info)?
        );

        Ok(RMQRedisManager {
            rmq_addr,
            rmq_port,
            redis_addr,
            redis_port,
            channel,
            frontier_queue,
            exchange,
            prefetch_count,
            redis_connection,
            redis_set,
        })
    }
}

impl Manager for RMQRedisManager {
    /// Submit a new task. The task must be checked if new before submission.
    fn submit_task(&self, task: &Task) -> ManagerResult<()> {
        let result = self
            .channel
            .basic_publish(
                self.exchange.as_str(),
                "",
                task.serialise(),
                BasicPublishOptions::default(),
                BasicProperties::default(),
            )
            .wait();

        result.map_err(|e| ManagerError::new(UnreachableError, "Could not reach manager.", Some(Box::new(e))))
    }

    /// Start resolving tasks with the given resolve function
    fn start_listening(&self, resolve_func: &dyn Fn(Task) -> TaskProcessResult) {
        self.channel
            .basic_consume(
                &self.frontier_queue,
                "",
                BasicConsumeOptions::default(),
                FieldTable::default(),
            )
            .and_then(move |consumer| {
                // Resolve each message received
                consumer.for_each(move |msg| {
                    match Task::deserialise(msg.data) {
                        Err(_) => {
                            // Deserialisation failed. Discard the task
                            self.channel.basic_reject(msg.delivery_tag, BasicRejectOptions::default())
                        }
                        Ok(task) => {
                            // Resolve task
                            match resolve_func(task) {
                                TaskProcessResult::Ok => {
                                    self.channel.basic_ack(msg.delivery_tag, false)
                                }
                                TaskProcessResult::Err => self
                                    .channel
                                    // Do not requeue task if error is met
                                    .basic_reject(msg.delivery_tag, BasicRejectOptions::default()),
                                TaskProcessResult::Reject => self
                                    .channel
                                    // Requeue task if error is met
                                    .basic_reject(msg.delivery_tag, BasicRejectOptions::default()),
                            }
                        }
                    }
                })
            })
            .wait()
            .unwrap();
    }

    /// Closes the manager and its connections
    fn close(self) -> ManagerResult<()> {
        self.channel.close(0, "Manager was closed by calling close()");
        Ok(())
    }

    /// Checks if a task has already been submitted
    fn contains(&self, task: &Task) -> ManagerResult<bool> {
        let mut con = self.redis_connection.lock().expect("Redis connection mutex was corruption");

        // Check if the task is a member of the collection
        con.sismember(self.redis_set.as_str(), task)
            .map_err(|e| ManagerError::new(UnreachableError, "Could not reach manager.", Some(Box::new(e))))
    }
}

/// Establishes a sentinel redis connection to the master group named 'master'
fn create_sentinel_redis_connection(connection_info: ConnectionInfo) -> Result<Connection, ()> {
    let mut client = redis::Client::open(connection_info.clone())
        .map_err(|_| ())?;

    // Get address and port of Redis master
    let (master_addr, master_port): (String, u16) = redis::cmd("SENTINEL")
        .arg("get-master-addr-by-name")
        .arg("master")
        .query(&mut client)
        .map_err(|_| ())?;

    let sentinel_client = redis::Client::open(
        ConnectionInfo {
            addr: Box::new(ConnectionAddr::Tcp(master_addr, master_port)),
            ..connection_info
        },
    ).map_err(|_| ())?;

    sentinel_client.get_connection()
        .map_err(|_| ())
}