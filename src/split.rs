use crate::traits::{Frontier, FrontierSubmitted, Submitted, TaskProcessResult};
use crate::Task;

struct SplitFrontierSubmitted<F: Frontier, S: Submitted> {
    frontier: F,
    submitted: S,
}

impl<F: Frontier, S: Submitted> SplitFrontierSubmitted<F, S> {
    fn new(frontier: F, submitted: S) -> Self {
        SplitFrontierSubmitted {
            frontier,
            submitted,
        }
    }
}

impl<F: Frontier, S: Submitted> FrontierSubmitted for SplitFrontierSubmitted<F, S> {
    fn submit_task(&self, task: &Task) -> Result<(), ()> {
        // Can we make any atomicity guarantees?
        self.frontier.submit_task(task)?;
        self.submitted.submit_task(task)
    }

    fn start_listening<G>(&self, f: G)
    where
        G: Fn(&Task) -> TaskProcessResult,
    {
        self.frontier.start_listening(f)
    }

    fn close(self) -> Result<(), ()> {
        self.frontier.close()
    }

    fn contains(&self, task: &Task) -> Result<bool, ()> {
        self.submitted.contains(task)
    }
}
