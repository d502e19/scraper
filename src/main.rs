extern crate rand;
extern crate futures;
extern crate tokio;
extern crate lapin_futures;

mod frontier;
mod task;

use crate::task::Task;
use crate::frontier::{RabbitmqFrontier, Frontier, TaskProcessResult};
use std::error::Error;
use std::thread::sleep;
use std::time::Duration;
use rand::Rng;

fn main() -> Result<(), Box<dyn Error>> {
    let addr = "amqp://192.168.99.100:5672/%2f";
    let seed = Task { url: String::from("https://aau.dk") };

    let frontier = RabbitmqFrontier::new(addr.to_string()).unwrap();
    frontier.submit_task(seed).unwrap();

    frontier.start_listening(Box::from(|task: Task| {
        println!("Received task: {}", task.url);

        // Simulate processing time
        let mut rng = rand::thread_rng();
        let process_time = rng.gen_range(50, 800);
        sleep(Duration::from_millis(process_time));

        // Spawn a random amount of new tasks
        let new_task_count = rng.gen_range(0, 4);
        for i in 0..new_task_count {
            let new_task = Task { url: format!("{}/{}", task.url, i) };
            frontier.submit_task(new_task).expect("Failed to submit task");
        }

        println!("Task took {}ms to process and spawn {} new tasks", process_time, new_task_count);

        TaskProcessResult::Ok
    }));

    frontier.close().expect("Could not close subscription");

    Ok(())
}
