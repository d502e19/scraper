extern crate futures;
extern crate lapin_futures;
extern crate rand;
extern crate redis;
extern crate tokio;

mod rmqredis;
mod split;
mod task;
mod traits;
mod worker;

use crate::task::Task;
use redis::Commands;
use std::collections::HashSet;
use std::error::Error;
use crate::worker::Worker;
use crate::rmqredis::RMQRedisManager;

fn main() -> Result<(), Box<dyn Error>> {
    /* // Construct a worker and its components
    let manager = RMQRedisManager::new(
        "192.162.99.100".to_string(),
        5672,
        6379,
        "work".to_string(),
        "".to_string(),
        "frontier".to_string(),
        "collection".to_string(),
    ).expect("Failed to construct RMQRedisManager");

    let worker = Worker::new(
        "Worker1".to_string(),
        manager,
        downloader, // TODO
        extractor, // TODO
        archive, // TODO
    );
    worker.start();
    */
    println!("Hello from worker!");
    Ok(())
}
