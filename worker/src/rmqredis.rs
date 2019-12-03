use std::sync::{Mutex};

use futures::future::Future;
use futures::stream::Stream;
use lapin_futures::{BasicProperties, Channel, Client, ConnectionProperties, ExchangeKind, Queue};
use lapin_futures::options::{
    BasicConsumeOptions, BasicPublishOptions, BasicRejectOptions, ExchangeDeclareOptions,
    QueueBindOptions, QueueDeclareOptions, BasicQosOptions,
};
use lapin_futures::types::FieldTable;
use redis::{Connection, ConnectionAddr, FromRedisValue, RedisError, RedisWrite, ToRedisArgs, Value, ConnectionInfo, IntoConnectionInfo, RedisResult, PipelineCommands};

use crate::errors::{ManagerError, ManagerResult};
use crate::errors::ManagerErrorKind::UnreachableError;
use crate::task::Task;
use crate::traits::{Manager, TaskProcessResult};
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::ops::DerefMut;
use std::borrow::BorrowMut;

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
                    .map_err(|_| RedisError::from(std::io::Error::new(std::io::ErrorKind::Other, "Failed to deserialise task")))
            }
            _ => Err(RedisError::from(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Response could not be translated to a task",
            )))
        }
    }
}

/// Error that encapsulates potential errors during construction of a RMQRedisManager.
/// It enables us to return detailed error messages.
#[derive(Debug)]
pub enum RMQRedisManagerError {
    RedisError(RedisError),
    RabbitMQError(lapin_futures::Error),
}

impl Error for RMQRedisManagerError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            RMQRedisManagerError::RedisError(e) => Some(e),
            RMQRedisManagerError::RabbitMQError(e) => Some(e),
        }
    }
}

impl Display for RMQRedisManagerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<RedisError> for RMQRedisManagerError {
    fn from(e: RedisError) -> Self {
        RMQRedisManagerError::RedisError(e)
    }
}

impl From<lapin_futures::Error> for RMQRedisManagerError {
    fn from(e: lapin_futures::Error) -> Self {
        RMQRedisManagerError::RabbitMQError(e)
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
        sentinel: Option<&str>,
    ) -> Result<RMQRedisManager, RMQRedisManagerError> {
        debug!("Creating RMQRedisManager with following values: \n\trmq_addr: {:?}\n\trmq_port: {:?}\
            \n\t redis_addr: {:?}\n\tredis_port: {:?}\n\trmq_exchange: {:?}\n\tprefetch_count: {:?}\
            \n\trmq_queue_name: {:?}\n\tcollection_queue_name: {:?}\n\tredis_set: {:?}\n\tsentinel: {:?}"
               , rmq_addr, rmq_port, redis_addr, redis_port, exchange, prefetch_count, frontier_queue_name, collection_queue_name, redis_set, sentinel);

        let client = Client::connect(
            format!("amqp://{}:{}/%2f", rmq_addr, rmq_port).as_str(),
            ConnectionProperties::default(),
        ).wait()?;

        let channel = client.create_channel().wait()?;

        let frontier_queue = channel.queue_declare(
            frontier_queue_name.as_str(),
            QueueDeclareOptions::default(),
            FieldTable::default(),
        ).wait()?;

        channel.queue_declare(
            collection_queue_name.as_str(),
            QueueDeclareOptions::default(),
            FieldTable::default(),
        ).wait()?;

        channel.exchange_declare(
            exchange.as_str(),
            ExchangeKind::Fanout,
            ExchangeDeclareOptions::default(),
            FieldTable::default(),
        ).wait()?;

        channel.queue_bind(
            frontier_queue_name.as_str(),
            exchange.as_str(),
            "",
            QueueBindOptions::default(),
            FieldTable::default(),
        ).wait()?;

        channel.queue_bind(
            collection_queue_name.as_str(),
            exchange.as_str(),
            "",
            QueueBindOptions::default(),
            FieldTable::default(),
        ).wait()?;

        // Limit the amount of tasks stored in the local queue
        channel.basic_qos(
            prefetch_count,
            BasicQosOptions::default(),
        ).wait()?;

        // Establish Redis connection
        let connection_info = format!("redis://{}:{}/", redis_addr, redis_port).as_str()
            .into_connection_info()?;
        let redis_connection = Mutex::new(
            create_redis_connection(connection_info, sentinel)?
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
    /// Submit new tasks. The tasks must be checked if new before submission.
    fn submit(&self, tasks: Vec<Task>) -> ManagerResult<()> {
        // Publish each task. If one fails, abort
        for task in tasks.iter() {
            let result = self.channel
                .basic_publish(
                    self.exchange.as_str(),
                    "",
                    task.serialise(),
                    BasicPublishOptions::default(),
                    BasicProperties::default(),
                )
                .wait();

            if let Err(e) = result {
                return Err(ManagerError::new(UnreachableError, "Could not reach manager.", Some(Box::new(e))))
            }
        }

        Ok(())
    }

    /// Start resolving tasks with the given resolve function
    fn subscribe(&self, resolve_func: &dyn Fn(Task) -> TaskProcessResult) {
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
                            info!("Discarded task due to failed deserialisation");
                            self.channel.basic_reject(msg.delivery_tag, BasicRejectOptions { requeue: false })
                        }
                        Ok(task) => {
                            // Resolve task
                            match resolve_func(task.clone()) {
                                TaskProcessResult::Ok => {
                                    self.channel.basic_ack(msg.delivery_tag, false)
                                }
                                TaskProcessResult::Err => {
                                    info!("Discarded task {}", task.url);
                                    self
                                        .channel
                                        // Do not requeue task if error is met
                                        .basic_reject(msg.delivery_tag, BasicRejectOptions { requeue: false })
                                },
                                TaskProcessResult::Reject => {
                                    info!("Rejected task {}", task.url);
                                    self
                                        .channel
                                        // Requeue task if error is met
                                        .basic_reject(msg.delivery_tag, BasicRejectOptions { requeue: false })
                                },
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
        // Redis connection does not have to be closed
        Ok(())
    }

    /// Cull tasks that have already been submitted once
    fn cull_known(&self, mut tasks: Vec<Task>) -> ManagerResult<Vec<Task>> {
        let mut con = self.redis_connection.lock().expect("Redis connection mutex was corrupted");

        // Query Redis about membership
        let reset_set = self.redis_set.as_str();
        let mut pipeline = redis::pipe();
        for task in tasks.iter() {
            pipeline.sismember(reset_set, task);
        }
        let is_member_vec: Vec<bool> = pipeline.query(con.deref_mut())
            .map_err(|e| ManagerError::new(UnreachableError, "Could not reach manager.", Some(Box::new(e))))?;

        // Remove those that are already members of the collection
        Ok(tasks.drain(..)
            .zip(is_member_vec)
            .filter_map(|(task, is_member)| {
                if !is_member {
                    Some(task)
                } else {
                    None
                }
            })
            .collect())
    }
}

/// Establishes a redis connection. An optional name of a master group can be given to
/// make the connection sentinel.
fn create_redis_connection(connection_info: ConnectionInfo, sentinel: Option<&str>) -> Result<Connection, RedisError> {
    let mut client = redis::Client::open(connection_info.clone())?;

    if let Some(name) = sentinel {
        // Get details about the Redis master
        let (master_addr, master_port) = redis::cmd("SENTINEL")
            .arg("get-master-addr-by-name")
            .arg(name)
            .query::<(String, u16)>(&mut client)?;

        // New sentinel client using master address and master port
        let sentinel_client = redis::Client::open(
            ConnectionInfo {
                addr: Box::new(ConnectionAddr::Tcp(master_addr, master_port)),
                ..connection_info
            },
        )?;

        return sentinel_client.get_connection()

    } else {
        // Non-sentinel connection
        return client.get_connection()
    }
}
