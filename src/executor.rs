use std::{
    collections::VecDeque,
    sync::{
        Arc, Condvar, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
    task::{Context, Poll},
};

use crate::{task::Task, waker::task_waker};

struct Inner {
    queue: VecDeque<Task>,
    alive: usize,
}

pub(crate) struct SharedState {
    inner: Mutex<Inner>,
    cvar: Condvar,
}

impl SharedState {
    pub(crate) fn add_task(&self, future: Task) {
        let mut inner = self.inner.lock().unwrap();
        inner.queue.push_back(future);
        drop(inner);

        self.cvar.notify_one();
    }

    pub(crate) fn add_alive(&self, add: usize) {
        let mut inner = self.inner.lock().unwrap();
        inner.alive += add;
    }
}

pub(crate) struct Executor {
    state: Arc<SharedState>,
}

impl Executor {
    pub(crate) fn new() -> Self {
        Self {
            state: Arc::new(SharedState {
                inner: Mutex::new(Inner {
                    queue: VecDeque::new(),
                    alive: 0,
                }),
                cvar: Condvar::new(),
            }),
        }
    }

    pub(crate) fn spawn(&mut self, future: impl Future<Output = ()> + 'static) {
        let task = Task::spawn(future);

        if task.mark_queued() {
            self.state.add_task(task);
            self.state.add_alive(1);
        }
    }

    pub(crate) fn run(&mut self) {
        let cvar = &self.state.cvar;
        let mut guard = self.state.inner.lock().unwrap();

        loop {
            while let Some(task) = guard.queue.pop_front() {
                task.clear_queued();

                let waker = task_waker(task.clone(), self.state.clone());
                let mut cx = Context::from_waker(&waker);

                drop(guard);
                let poll = task.poll(&mut cx);
                guard = self.state.inner.lock().unwrap();

                if poll == Poll::Ready(()) {
                    guard.alive -= 1;
                }
            }

            if guard.alive == 0 {
                break;
            }

            while guard.alive != 0 && guard.queue.is_empty() {
                guard = cvar.wait(guard).unwrap();
            }
        }
    }
}
