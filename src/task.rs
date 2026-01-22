use std::{
    cell::UnsafeCell,
    pin::Pin,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    task::{Context, Poll},
};

struct TaskInner {
    future: UnsafeCell<Option<Pin<Box<dyn Future<Output = ()>>>>>,
    queued: AtomicBool,
}

#[derive(Clone)]
pub(crate) struct Task {
    inner: Arc<TaskInner>,
}

impl Task {
    pub(crate) fn spawn(future: impl Future<Output = ()> + 'static) -> Self {
        Self {
            inner: Arc::new(TaskInner {
                future: UnsafeCell::new(Some(Box::pin(future))),
                queued: AtomicBool::new(false),
            }),
        }
    }

    pub(crate) fn mark_queued(&self) -> bool {
        self.inner
            .queued
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
    }

    pub(crate) fn clear_queued(&self) {
        self.inner.queued.store(false, Ordering::Release);
    }

    pub(crate) fn poll(&self, cx: &mut Context<'_>) -> Poll<()> {
        let ptr = self.inner.future.get();

        // SAFETY:
        // There is not other thread muteting the same future
        let fut = unsafe { &mut *ptr };

        let future = match fut {
            None => return Poll::Ready(()),
            Some(v) => v.as_mut(),
        };

        let poll = future.poll(cx);

        if poll == Poll::Ready(()) {
            *fut = None;
        }

        poll
    }
}
