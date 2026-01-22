use std::{
    pin::Pin,
    task::{Context, Poll},
};

use rand::Rng;

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

fn main() {
    let exe = Executor::new();

    for i in 0..2 {
        let mut rng = rand::rng();
        let n: u8 = rng.random_range(10..=100);

        exe.spawn(TestFuture::new(n, i));
    }

    exe.run();
}
