## Setup

So this is our initial setup. We just have a simple Async Runtime with just the `block_on` function which allows us to
execute futures:

```rust
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll, Wake};
use std::thread::Thread;

pub struct SimpleRuntime;

impl SimpleRuntime {
    pub fn block_on<F>(mut f: F) -> F::Output
    where
        F: Future,
    {
        let mut f = unsafe { Pin::new_unchecked(&mut f) };

        let thread = std::thread::current();
        let waker = Arc::new(SimpleWaker { thread }).into();
        let mut ctx = Context::from_waker(&waker);

        loop {
            println!("polling future");
            match f.as_mut().poll(&mut ctx) {
                Poll::Ready(val) => {
                    println!("future is ready");
                    return val;
                }
                Poll::Pending => {
                    std::thread::park();
                    println!("parked");
                }
            }
        }
    }
}

pub struct SimpleWaker {
    thread: Thread,
}

impl Wake for SimpleWaker {
    fn wake(self: std::sync::Arc<Self>) {
        self.thread.unpark();
    }
}
```

And we've got a Future which counts to a provided number:

```rust
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::AtomicU64;
use std::task::{Context, Poll};

pub struct Counter {
    pub counter: Arc<AtomicU64>,
    pub max: u64,
}

impl Future for Counter {
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
```

And we run it in our main:

```rust
mod runtime;
mod task;

use std::sync::Arc;
use std::sync::atomic::AtomicU64;

use runtime::SimpleRuntime;
use task::Counter;

fn main() {
    SimpleRuntime::block_on(async_main());
}

async fn async_main() {
    let res = count(10).await;
    println!("async_main {res}");
}

fn count(max: u64) -> impl Future<Output = u64> {
    println!("counting to {max}");
    Counter {
        counter: Arc::new(AtomicU64::new(0)),
        max,
    }
}
```
