use {
    super::Task,
    alloc::collections::VecDeque,
    core::task::{Context, Poll, RawWaker, RawWakerVTable, Waker},
};

pub struct SimpleExecutor {
    task_queue: VecDeque<Task>,
}

impl SimpleExecutor {
    pub fn new() -> Self {
        Self {
            task_queue: VecDeque::new(),
        }
    }

    pub fn spawn(&mut self, task: Task) {
        self.task_queue.push_back(task)
    }

    pub fn run(&mut self) {
        while let Some(mut task) = self.task_queue.pop_front() {
            match task.poll(&mut Context::from_waker(&dummy_waker())) {
                Poll::Ready(()) => {}
                Poll::Pending => self.task_queue.push_back(task),
            }
        }
    }
}

fn dummy_raw_waker() -> RawWaker {
    fn no_op(_: *const ()) {}

    fn clone(_: *const ()) -> RawWaker {
        dummy_raw_waker()
    }

    RawWaker::new(
        0 as *const (),
        &RawWakerVTable::new(clone, no_op, no_op, no_op),
    )
}

fn dummy_waker() -> Waker {
    unsafe { Waker::from_raw(dummy_raw_waker()) }
}
