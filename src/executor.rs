use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
    task::Context,
};

use crate::{task::Task, waker::task_waker};

pub(crate) struct SharedQueue(Mutex<VecDeque<Task>>);

impl SharedQueue {
    pub(crate) fn push(&self, future: Task) {
        self.0.lock().unwrap().push_back(future);
    }

    pub(crate) fn pop(&self) -> Option<Task> {
        self.0.lock().unwrap().pop_front()
    }
}

pub(crate) struct Executor {
    queue: Arc<SharedQueue>,
}

impl Executor {
    pub(crate) fn new() -> Self {
        Self {
            queue: Arc::new(SharedQueue(Mutex::new(VecDeque::new()))),
        }
    }

    pub(crate) fn spawn(&self, future: impl Future<Output = ()> + 'static) {
        let task = Task::spawn(future);

        if task.mark_queued() {
            self.queue.push(task);
        }
    }

    pub(crate) fn run(&self) {
        while let Some(task) = self.queue.pop() {
            let waker = task_waker(task.clone(), self.queue.clone());
            let mut cx = Context::from_waker(&waker);

            task.clear_queued();
            let _ = task.poll(&mut cx);
        }
    }
}
