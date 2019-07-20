use std::task;

pub struct Context<'a> {
    // used to wake the actor's task.
    task: task::Context<'a>,
    // ...
}

impl<'a> Context<'a> {
    /// Returns a reference to the `Waker` for the current task.
    #[inline]
    pub fn waker(&self) -> &'a task::Waker {
        &self.waker
    }
}
