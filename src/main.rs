extern crate futures;
extern crate tokio;
extern crate lapin_futures;

mod frontier;
mod task;

use crate::task::Task;
use crate::frontier::{RabbitmqFrontier, Frontier, TaskSubmitResult};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let addr = "amqp://192.168.99.100:5672/%2f";

    let msg = Task { url: String::from("https://aau.dk") };

    let frontier = RabbitmqFrontier::new(addr.to_string()).unwrap();
    frontier.submit_task(msg).unwrap();

    let f = |msg: Task| {
        println!("Task has url: {}", msg.url);
        TaskSubmitResult::Ok
    };

    frontier.subscribe(Box::from(f));

    frontier.close().expect("Could not close subscription");

    Ok(())
}
