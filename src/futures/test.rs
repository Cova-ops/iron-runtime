use std::{
    pin::Pin,
    task::{Context, Poll},
};

pub(crate) struct TestFuture {
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
