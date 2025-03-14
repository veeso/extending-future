use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::task::{Context, Poll, Wake};
use std::thread::Thread;

use crate::FutureExt;

pub struct SimpleRuntime {
    abort: Arc<AtomicBool>,
}

impl SimpleRuntime {
    pub fn new(abort: &Arc<AtomicBool>) -> Self {
        Self {
            abort: Arc::clone(abort),
        }
    }

    pub fn block_on<F>(&self, mut f: F) -> F::Output
    where
        F: FutureExt,
    {
        let mut f = unsafe { Pin::new_unchecked(&mut f) };

        let thread = std::thread::current();
        let waker = Arc::new(SimpleWaker { thread }).into();
        let mut ctx = Context::from_waker(&waker);

        loop {
            if self.abort.load(std::sync::atomic::Ordering::Relaxed) {
                println!("aborting future");
                f.abort();
            }

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
