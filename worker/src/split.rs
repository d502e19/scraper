use crate::errors::ManagerResult;
use crate::Task;
use crate::traits::{Collection, Frontier, Manager, TaskProcessResult};

/// A SplitManager is a manager that consists of a separate frontier and Collection. I.e. this can
/// be used then there are no dependencies between the frontier implementation and the
/// collection implementation.
/// When a task is submitted, it is submitted to the frontier first and secondly the collection.
/// Since submitting to the frontier and the collection can fail, the task is not guaranteed to
/// submitted to the collection, even if was successfully submitted to the frontier.
#[allow(dead_code)]
struct SplitManager {
    frontier: Box<dyn Frontier>,
    collection: Box<dyn Collection>,
}

#[allow(dead_code)]
impl SplitManager {
    /// Construct a new SplitManager with the given Frontier and Collection
    fn new(frontier: Box<dyn Frontier>, collection: Box<dyn Collection>) -> Self {
        SplitManager {
            frontier,
            collection,
        }
    }
}

impl Manager for SplitManager {
    /// Submits a task to the Frontier and the Collection.
    /// When a task is submitted, it is submitted to the frontier first and secondly the collection.
    /// Since submitting to the frontier and the collection can fail, the task is not guaranteed to
    /// submitted to the collection, even if was successfully submitted to the frontier.
    fn submit_task(&self, task: &Task) -> ManagerResult<()> {
        self.frontier.submit_task(task)?;
        self.collection.submit_task(task)
    }

    /// Starts resolving Task with the given resolve function
    fn start_listening(&self, resolve_func: &dyn Fn(Task) -> TaskProcessResult) {
        self.frontier.start_listening(resolve_func)
    }

    /// Closes the SplitManager and any open connections that the Frontier may have
    fn close(self) -> ManagerResult<()> {
        self.frontier.close()?;
        Ok(())
    }

    /// Checks if a Task has already been submitted
    fn contains(&self, task: &Task) -> ManagerResult<bool> {
        self.collection.contains(task)
    }
}
