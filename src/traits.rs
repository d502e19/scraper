use crate::task::Task;

pub(crate) trait FrontierSubmitted {
    fn submit_task(&self, task: &Task) -> Result<(), ()>;

    fn start_listening<F>(&self, f: F)
        where
            F: Fn(&Task) -> TaskProcessResult;

    fn close(self) -> Result<(), ()>;

    fn contains(&self, task: &Task) -> Result<bool, ()>;
}

pub trait Frontier {
    fn submit_task(&self, task: &Task) -> Result<(), ()>;

    fn start_listening<F>(&self, f: F)
        where
            F: Fn(&Task) -> TaskProcessResult;

    fn close(self) -> Result<(), ()>;
}

pub enum TaskProcessResult {
    Ok,
    Err,
    Reject,
}

pub trait Submitted {
    fn contains(&self, task: &Task) -> Result<bool, ()>;

    fn submit_task(&self, task: &Task) -> Result<(), ()>;
}

