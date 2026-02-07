use crate::{executor::Executor, futures::sleep::SleepFuture};

mod executor;
mod futures;
mod task;
mod waker;

fn main() {
    let mut exe = Executor::new();

    println!("Start");
    exe.spawn(SleepFuture::new(1000));
    exe.spawn(SleepFuture::new(200));
    exe.spawn(SleepFuture::new(100));
    exe.run();
    println!("End");
}
