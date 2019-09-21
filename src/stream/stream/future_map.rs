use std::pin::Pin;

use crate::future::Future;
use crate::stream::Stream;
use crate::task::{Context, Poll};

/// Stream for the [`future_map`] method.
///
/// [`future_map`]: super::Stream::future_map
#[derive(Debug)]
pub struct FutureMap<S, F, Fut> {
    stream: S,
    f: F,
    fut: Option<Fut>,
}

impl<S: Unpin, F, Fut: Unpin> Unpin for FutureMap<S, F, Fut> {}

impl<'a, S, F, Fut> FutureMap<S, F, Fut>
where
    S: 'a + Stream,
    F: 'a + FnMut(S::Item) -> Fut,
    Fut: 'a + Future,
{
    pub(crate) fn new(stream: S, f: F) -> Self {
        Self {
            stream,
            f,
            fut: None,
        }
    }

    pin_utils::unsafe_pinned!(stream: S);
    pin_utils::unsafe_unpinned!(f: F);
    pin_utils::unsafe_pinned!(fut: Option<Fut>);
}

impl<'a, S, F, Fut, U> futures_core::stream::Stream for FutureMap<S, F, Fut>
where
    S: 'a + Stream,
    F: 'a + FnMut(S::Item) -> Fut,
    Fut: 'a + Future<Output = U>,
{
    type Item = U;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<U>> {
        if let Some(fut) = self.as_mut().fut().as_pin_mut() {
            let item = futures_core::ready!(fut.poll(cx));
            self.as_mut().fut().set(None);
            return Poll::Ready(Some(item));
        }
        let item = futures_core::ready!(self.as_mut().stream().poll_next(cx));
        if let Some(item) = item {
            let fut = (self.as_mut().f())(item);
            self.as_mut().fut().set(Some(fut));
            cx.waker().wake_by_ref();
            Poll::Pending
        } else {
            Poll::Ready(None)
        }
    }
}
