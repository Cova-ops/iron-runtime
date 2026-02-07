use std::{
    pin::Pin,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    task::{Context, Poll, Waker},
    time::{Duration, Instant},
};

struct StateSleepFuture {
    completed: AtomicBool,
    waker: Mutex<Option<Waker>>,
}

pub(crate) struct SleepFuture {
    state: Arc<StateSleepFuture>,
    started: bool,
    time_sleep: Duration,
    deadline: Instant,
}

impl SleepFuture {
    pub(crate) fn new(ms: u64) -> Self {
        Self {
            state: Arc::new(StateSleepFuture {
                completed: AtomicBool::new(false),
                waker: Mutex::new(None),
            }),
            started: false,
            time_sleep: Duration::from_millis(ms),
            deadline: Instant::now()
                .checked_add(Duration::from_millis(ms))
                .unwrap(),
        }
    }
}

impl Future for SleepFuture {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.state.completed.load(Ordering::Acquire) {
            println!("Sleep {}ms. Done!", self.time_sleep.as_millis());

            Poll::Ready(())
        } else if Instant::now() >= self.deadline {
            self.state.completed.store(true, Ordering::Release);
            println!("Sleep {}ms. Done!", self.time_sleep.as_millis());

            Poll::Ready(())
        } else if self.started {
            let mut guard = self.state.waker.lock().unwrap();
            *guard = Some(cx.waker().clone());

            Poll::Pending
        } else {
            self.started = true;

            let mut guard = self.state.waker.lock().unwrap();
            *guard = Some(cx.waker().clone());

            let state2 = self.state.clone();
            let duration = self.time_sleep;

            std::thread::spawn(move || {
                std::thread::sleep(duration);
                state2.completed.store(true, Ordering::Release);

                let waker = {
                    let mut guard = state2.waker.lock().unwrap();
                    guard.take()
                };

                match waker {
                    Some(v) => v.wake(),
                    _ => {}
                }
            });

            Poll::Pending
        }
    }
}

#[cfg(test)]
mod test_unit_sleep_thread {

    use crate::waker::test_waker::noop_waker;

    use super::*;

    #[test]
    fn pending_first_poll() {
        let mut sleep_f = SleepFuture::new(50);
        let sleep_f = Pin::new(&mut sleep_f);

        let waker = noop_waker();
        let mut cx = Context::from_waker(&waker);

        let poll = sleep_f.poll(&mut cx);
        assert_eq!(
            poll,
            Poll::Pending,
            "The first poll always need to return a Poll::Pending"
        );
    }

    #[test]
    fn wait_time() {
        let mut sleep_f = SleepFuture::new(50);
        let mut sleep_f = Pin::new(&mut sleep_f);

        let waker = noop_waker();
        let mut cx = Context::from_waker(&waker);

        let _ = sleep_f.as_mut().poll(&mut cx);
        std::thread::sleep(Duration::from_millis(100));

        let poll = sleep_f.poll(&mut cx);
        assert_eq!(poll, Poll::Ready(()), "The thread should finished");
    }
}
