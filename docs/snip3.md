## Future Adapter

We can actually fix this issue by using a trick! We can create a new **trait** that will be implemented for `Future` that will convert it to `FutureExt`:

```rust
/// A trait for adapting a [`Future`] into a [`FutureExt`].
///
/// With this we can convert any [`Future`] into a [`FutureExt`] by calling the `adapt` method.
///
/// # Example
///
/// ```
/// use ext_fut::FutureAdapter;
/// use std::future::Future;
///
/// async fn foo() {}
///
/// let fut = foo().adapt();
/// ```
pub trait FutureAdapter: Future {
    fn adapt(self) -> impl FutureExt<Output = Self::Output>
    where
        Self: Sized,
    {
        FutureWrapper { inner: self }
    }
}

impl<F: Future> FutureAdapter for F {}

/// Internal struct which wraps a [`Future`] and implements [`FutureExt`].
struct FutureWrapper<F> {
    inner: F,
}

impl<F: Future> Future for FutureWrapper<F> {
    type Output = F::Output;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        unsafe { self.as_mut().map_unchecked_mut(|s| &mut s.inner) }.poll(cx)
    }
}
```

And now we can use it in our `SimpleRuntime`:

```rust
use self::ext_fut::FutureExt;
use self::runtime::SimpleRuntime;
use self::task::{count, permute};

fn main() {
    SimpleRuntime::block_on(async_main().adapt());
    let permutations = SimpleRuntime::block_on(permute(&[1, 6, 4, 3, 2, 5], &[1, 2, 3, 4, 5, 6]));
    println!("permutations: {permutations}");
}
```

And with that we can finally have the runtime to run **std Futures as well**.
