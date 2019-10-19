extern crate futures;
extern crate lapin_futures;
extern crate rand;
extern crate redis;
extern crate tokio;

mod rmqredis;
mod split;
mod task;
mod traits;
mod void;
mod archive;

use crate::task::Task;
use redis::Commands;
use std::collections::HashSet;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let client = redis::Client::open("redis://192.168.99.100:6379/").unwrap();
    let con_result = client.get_connection();
    match con_result {
        Ok(mut con) => {
            let task: Task = Task {
                url: String::from("aau.dk"),
            };
            let task2: Task = Task {
                url: String::from("wikipedia.dk"),
            };

            // Submit tasks to Redis
            let _: () = con.sadd("submitted", &task)?;
            let _: () = con.sadd("submitted", &task2)?;

            // Get tasks from Redis
            let result: HashSet<Task> = con.smembers("submitted")?;
            println!("Redis contained: {:?}", result);

            let found: bool = con.sismember("submitted", &task)?;
            println!("{} is in set: {}", task.url, found);
        }
        Err(_) => println!("Couldn't connect to Redis."),
    }

    /*
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
    */
    Ok(())
}
