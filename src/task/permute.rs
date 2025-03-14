use std::pin::Pin;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};
use std::time::Duration;

use crate::FutureExt;

/// A future that at each poll will advance in the permutation of a list of numbers.
///
/// Given a list of numbers, it permutates them until it reaches the target permutation.
///
/// Returns the amount of steps required
struct PermuteFuture {
    current: Arc<Mutex<Vec<u64>>>,
    target: Vec<u64>,
    steps: AtomicU64,
    aborted: AtomicBool,
}

impl PermuteFuture {
    pub fn new(base: &[u64], target: &[u64]) -> Self {
        if base.len() != target.len() {
            panic!("Both lists must have the same length");
        }
        if base.iter().any(|&x| !target.contains(&x)) {
            panic!("Both lists must have the same elements");
        }

        Self {
            current: Arc::new(Mutex::new(base.to_vec())),
            target: target.to_vec(),
            steps: AtomicU64::new(0),
            aborted: AtomicBool::new(false),
        }
    }

    /// Execute one step of the permutation
    fn permute(&self) {
        let mut current = self.current.lock().unwrap();

        let mut swap: (Option<_>, Option<_>) = (None, None);
        // find the first number that can be swapped
        for i in 0..current.len() {
            if current[i] != self.target[i] {
                swap.0 = Some(i);
                break;
            }
        }
        if swap.0.is_none() {
            return;
        }

        // the second is the number that should be in the position of the first
        for j in 0..current.len() {
            if current[j] == self.target[swap.0.unwrap()] {
                swap.1 = Some(j);
                break;
            }
        }

        // swap the two numbers
        if let (Some(i), Some(j)) = swap {
            current.swap(i, j);
        }

        self.steps.fetch_add(1, Ordering::Relaxed);
    }
}

impl FutureExt for PermuteFuture {
    fn abort(&self) {
        self.aborted.store(true, Ordering::Relaxed);
    }
}

impl Future for PermuteFuture {
    type Output = u64;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.aborted.load(Ordering::Relaxed) {
            return Poll::Ready(self.steps.load(Ordering::Relaxed));
        }

        std::thread::sleep(Duration::from_secs(2));

        self.permute();
        let current = self.current.lock().unwrap();
        println!("current {current:?}, target {:?}", self.target);

        if *current == self.target {
            Poll::Ready(self.steps.load(Ordering::Relaxed))
        } else {
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}

pub fn permute(base: &[u64], target: &[u64]) -> impl FutureExt<Output = u64> {
    PermuteFuture::new(base, target)
}
