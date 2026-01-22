use std::{
    sync::Arc,
    task::{RawWaker, RawWakerVTable, Waker},
};

use crate::{executor::SharedQueue, task::Task};

static VTABLE: RawWakerVTable =
    RawWakerVTable::new(clone_waker, wake_waker, wake_by_ref_waker, drop_waker);

pub(crate) struct WakerData {
    task: Task,
    pub(super) queue: Arc<SharedQueue>,
}

unsafe fn wake_waker(ptr: *const ()) {
    // SAFETY: `ptr` was created from `Arc<WakerData>` via `Arc::into_raw`
    // and is valid for the lifetime of this waker.
    let arc = unsafe { Arc::from_raw(ptr as *const WakerData) };

    if arc.task.mark_queued() {
        arc.queue.push(arc.task.clone());
    }
}

unsafe fn wake_by_ref_waker(ptr: *const ()) {
    // SAFETY: `ptr` was created from `Arc<WakerData>` via `Arc::into_raw`
    // and is valid for the lifetime of this waker.
    let arc = unsafe { Arc::from_raw(ptr as *const WakerData) };

    if arc.task.mark_queued() {
        arc.queue.push(arc.task.clone());
    }

    // Don't drop the Arc; restore the raw pointer ownership.
    let _ = Arc::into_raw(arc);
}

unsafe fn drop_waker(ptr: *const ()) {
    // SAFETY: `ptr` was created from `Arc<WakerData>` via `Arc::into_raw`.
    // This function is called exactly once for each RawWaker instance, so it is
    // safe to reconstruct and drop one strong reference.
    drop(unsafe { Arc::from_raw(ptr as *const WakerData) });
}

unsafe fn clone_waker(ptr: *const ()) -> RawWaker {
    // SAFETY: `ptr` was created from `Arc<WakerData>` via `Arc::into_raw`
    // and is valid for the lifetime of this waker.
    let arc = unsafe { Arc::from_raw(ptr as *const WakerData) };
    let arc_clone = arc.clone();
    let _ = Arc::into_raw(arc);

    let ptr = Arc::into_raw(arc_clone) as *const ();
    RawWaker::new(ptr, &VTABLE)
}

pub(crate) fn task_waker(task: Task, queue: Arc<SharedQueue>) -> Waker {
    let waker_data = Arc::new(WakerData { task, queue });
    let ptr = Arc::into_raw(waker_data) as *const ();
    let raw = RawWaker::new(ptr, &VTABLE);

    // SAFETY: ptr was produced by Arc::into_raw and VTABLE functions
    // uphold the RawWaker contract for clone/wake/wake_by_ref/drop.
    unsafe { Waker::from_raw(raw) }
}
