#[cfg(test)]
mod test_sleep_future {
    struct WrapperSleepFuture {
        inner: SleepFuture,
        id: u32,
        log: Arc<Mutex<Vec<u32>>>,
        done: bool,
    }
}
