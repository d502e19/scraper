extern crate futures;
extern crate tokio;
extern crate lapin_futures;

mod frontier;
mod task;

use crate::task::Task;
use crate::frontier::{RabbitmqFrontier, Frontier, TaskProcessResult};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let addr = "amqp://192.168.99.100:5672/%2f";
    let seed = Task { url: String::from("https://aau.dk") };

    let frontier = RabbitmqFrontier::new(addr.to_string()).unwrap();
    frontier.submit_task(seed).unwrap();

    frontier.start_listening(Box::from(|task: Task| {
        println!("Processing task: {}", task.url);

        let first_new_task = Task { url: format!("{}/0", task.url) };
        let second_new_task = Task { url: format!("{}/1", task.url) };

        frontier.submit_task(first_new_task).expect("Failed to submit task");
        frontier.submit_task(second_new_task).expect("Failed to submit task");

        TaskProcessResult::Ok
    }));

    frontier.close().expect("Could not close subscription");

    Ok(())
}
