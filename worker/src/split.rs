use crate::errors::ManagerResult;
use crate::Task;
use crate::traits::{Collection, Frontier, Manager, TaskProcessResult};

/// A SplitManager is a manager that consists of a separate frontier and collection. I.e. this can
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
    /// Submit tasks to the Frontier and the Collection.
    /// When a task is submitted, it is submitted to the frontier first and secondly the collection.
    /// Since submitting to the frontier and the collection can fail, the task is not guaranteed to
    /// submitted to the collection, even if was successfully submitted to the frontier.
    /// The tasks must be checked if new before submission.
    fn submit(&self, tasks: Vec<Task>) -> ManagerResult<()> {
        self.frontier.submit(tasks.clone())?;
        self.collection.submit(tasks)
    }

    /// Starts resolving tasks with the given resolve function
    fn subscribe(&self, resolve_func: &dyn Fn(Task) -> TaskProcessResult) {
        self.frontier.subscribe(resolve_func)
    }

    /// Closes the SplitManager and any open connections that it may have
    fn close(self) -> ManagerResult<()> {
        self.frontier.close()?;
        self.collection.close()
    }

    /// Checks if a Task has already been submitted
    fn cull_known(&self, tasks: Vec<Task>) -> ManagerResult<Vec<Task>> {
        self.collection.cull_known(tasks)
    }
}
