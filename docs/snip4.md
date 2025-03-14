## FutureExt in action

Of course, at the moment our `FutureExt` is doing nothing, but we can change our Runtime to actually use our `abort` method.

We can for example make it to call `abort()` whenever ctrl+c is pressed:

First of all we change our runtime to take an `AtomicBool` to abort:

```rust
pub struct SimpleRuntime {
    abort: Arc<AtomicBool>,
}

impl SimpleRuntime {
    pub fn new(abort: &Arc<AtomicBool>) -> Self {
        Self {
            abort: Arc::clone(abort),
        }
    }
```

And we change `block_on` to take `&self` and to abort in case it's true during future execution:

```rust
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
```

Let's add the handler in our main:

```rust
fn main() {
    let abort = Arc::new(AtomicBool::new(false));
    let runtime = SimpleRuntime::new(&abort);

    // setup ctrlc to abort
    let abort_clone = Arc::clone(&abort);
    ctrlc::set_handler(move || {
        abort_clone.store(true, std::sync::atomic::Ordering::Relaxed);
    })
    .expect("Error setting Ctrl-C handler");

    runtime.block_on(async_main().adapt());
    let permutations = runtime.block_on(permute(&[1, 6, 4, 3, 2, 5], &[1, 2, 3, 4, 5, 6]));
    println!("permutations: {permutations}");
}
```

And finally I've added a `sleep` in our `PermuteFuture` to simulate a long running future, in order to manage to abort.

Let's run it!

```txt
parked
polling future
^Ccurrent [1, 2, 3, 4, 6, 5], target [1, 2, 3, 4, 5, 6]
parked
aborting future
polling future
future is ready
permutations: 2
```

So after pressing ctrl+c, it actually aborted the execution. Cool!
