use crate::errors::ManagerResult;
use crate::Task;
use crate::traits::{Collection, Frontier, Manager, TaskProcessResult};

struct SplitManager<F: Frontier, S: Collection> {
    frontier: F,
    collection: S,
}

impl<F: Frontier, S: Collection> SplitManager<F, S> {
    fn new(frontier: F, collection: S) -> Self {
        SplitManager {
            frontier,
            collection,
        }
    }
}

impl<F: Frontier, S: Collection> Manager for SplitManager<F, S> {
    fn submit_task(&self, task: &Task) -> ManagerResult<()> {
        // Can we make any atomicity guarantees?
        self.frontier.submit_task(task)?;
        self.collection.submit_task(task)
    }

    fn start_listening<G>(&self, f: G)
    where
        G: Fn(Task) -> TaskProcessResult,
    {
        self.frontier.start_listening(f)
    }

    fn close(self) -> ManagerResult<()> {
        self.frontier.close()
    }

    fn contains(&self, task: &Task) -> ManagerResult<bool> {
        self.collection.contains(task)
    }
}
