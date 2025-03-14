//! Extension of Future

use std::pin::Pin;
use std::task::{Context, Poll};

/// An extension of future which provides additional methods for working with futures.
pub trait FutureExt: Future {
    /// Abort the future.
    fn abort(&self);
}

impl<F: Future> FutureExt for FutureWrapper<F> {
    fn abort(&self) {
        // no-op
    }
}

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
