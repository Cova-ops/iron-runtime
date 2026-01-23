use std::{
    pin::Pin,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    task::{Context, Poll, Waker},
    time::{Duration, Instant, SystemTime},
};

use crate::executor::Executor;

mod executor;
mod task;
mod waker;

struct TestFuture {
    remaining: u8,
    id: u8,
}

impl TestFuture {
    fn new(remaining: u8, id: u8) -> Self {
        Self { remaining, id }
    }
}

impl Future for TestFuture {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.remaining == 0 {
            println!("TestFuture: Ready");
            Poll::Ready(())
        } else {
            println!(
                "TestFuture: Pending, remaining = {}, id = {}",
                self.remaining, self.id
            );

            self.remaining -= 1;
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}

struct StateSleepFuture {
    completed: AtomicBool,
    waker: Mutex<Option<Waker>>,
}

struct SleepFuture {
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

fn main() {
    let mut exe = Executor::new();

    println!("Start");
    exe.spawn(SleepFuture::new(1000));
    exe.spawn(SleepFuture::new(200));
    exe.spawn(SleepFuture::new(100));
    exe.run();
    println!("End");
}
