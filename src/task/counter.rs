use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::AtomicU64;
use std::task::{Context, Poll};

pub struct CounterFuture {
    pub counter: Arc<AtomicU64>,
    pub max: u64,
}

impl Future for CounterFuture {
    type Output = u64;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.counter
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);

        let value = self.counter.load(std::sync::atomic::Ordering::SeqCst);
        println!("polled with current value: {value}");

        if value >= self.max {
            Poll::Ready(value)
        } else {
            // wake up the future
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}

pub fn count(max: u64) -> impl Future<Output = u64> {
    println!("counting to {max}");
    CounterFuture {
        counter: Arc::new(AtomicU64::new(0)),
        max,
    }
}
