use std::pin::Pin;

use crate::prelude::*;
use crate::stream::IntoStream;

/// Extend a collection with the contents of a stream.
///
/// Streams produce a series of values asynchronously, and collections can also be thought of as a
/// series of values. The `Extend` trait bridges this gap, allowing you to extend a collection
/// asynchronously by including the contents of that stream. When extending a collection with an
/// already existing key, that entry is updated or, in the case of collections that permit multiple
/// entries with equal keys, that entry is inserted.
///
/// ## Examples
///
/// ```
/// # fn main() { async_std::task::block_on(async {
/// #
/// use async_std::prelude::*;
/// use async_std::stream::{self, Extend};
///
/// let mut v: Vec<usize> = vec![1, 2];
/// let s = stream::repeat(3usize).take(3);
/// v.stream_extend(s).await;
///
/// assert_eq!(v, vec![1, 2, 3, 3, 3]);
/// #
/// # }) }
/// ```
#[cfg_attr(feature = "docs", doc(cfg(unstable)))]
pub trait Extend<A> {
    /// Extends a collection with the contents of a stream.
    fn stream_extend<'a, T: IntoStream<Item = A> + 'a>(
        &'a mut self,
        stream: T,
    ) -> Pin<Box<dyn Future<Output = ()> + 'a>>;
}

impl Extend<()> for () {
    fn stream_extend<'a, T: IntoStream<Item = ()> + 'a>(
        &'a mut self,
        stream: T,
    ) -> Pin<Box<dyn Future<Output = ()> + 'a>> {
        let stream = stream.into_stream();
        Box::pin(async move {
            pin_utils::pin_mut!(stream);
            while let Some(_) = stream.next().await {}
        })
    }
}

/// An adapter that makes `Sink`s implement [`Extend`].
///
/// [`Extend`]: crate::stream::Extend
///
/// ## Examples
///
/// ```
/// # fn main() { async_std::task::block_on(async {
/// #
/// use async_std::prelude::*;
/// use async_std::stream::{self, Extend, ExtendSink};
///
/// let mut v: Vec<usize> = vec![1, 2];
/// let s = stream::repeat(3usize).take(3);
/// let mut sink = ExtendSink::new(&mut v);
/// sink.stream_extend(s).await;
///
/// assert_eq!(v, vec![1, 2, 3, 3, 3]);
/// #
/// # }) }
/// ```
#[cfg_attr(feature = "docs", doc(cfg(unstable)))]
#[derive(Debug)]
pub struct ExtendSink<S>(S);

impl<S> ExtendSink<S> {
    /// Create an adapter which implements `Extend` from a sink.
    #[cfg_attr(feature = "docs", doc(cfg(unstable)))]
    pub fn new(sink: S) -> Self {
        Self(sink)
    }

    /// Consume this adapter, and return the inner sink.
    #[cfg_attr(feature = "docs", doc(cfg(unstable)))]
    pub fn into_inner(self) -> S {
        self.0
    }
}

impl<S, A> Extend<A> for ExtendSink<S>
where
    S: futures_sink::Sink<A> + Unpin,
{
    fn stream_extend<'a, T: IntoStream<Item = A> + 'a>(
        &'a mut self,
        stream: T,
    ) -> Pin<Box<dyn Future<Output = ()> + 'a>> {
        let mut sink = Pin::new(&mut self.0);
        let stream = stream.into_stream();
        Box::pin(async move {
            pin_utils::pin_mut!(stream);
            while let Some(item) = stream.next().await {
                let _ = crate::future::poll_fn(|cx| sink.as_mut().poll_ready(cx)).await;
                let _ = sink.as_mut().start_send(item);
            }
            let _ = crate::future::poll_fn(|cx| sink.as_mut().poll_flush(cx)).await;
        })
    }
}

/// Create an adapter which implements [`Extend`] from a sink.
///
/// [`Extend`]: crate::stream::Extend
///
/// ## Examples
///
/// ```
/// # fn main() { async_std::task::block_on(async {
/// #
/// use async_std::prelude::*;
/// use async_std::stream::{self, Extend};
///
/// let mut v: Vec<usize> = vec![1, 2];
/// let s = stream::repeat(3usize).take(3);
/// let mut sink = stream::extend_sink(&mut v);
/// sink.stream_extend(s).await;
///
/// assert_eq!(v, vec![1, 2, 3, 3, 3]);
/// #
/// # }) }
/// ```
#[cfg_attr(feature = "docs", doc(cfg(unstable)))]
pub fn extend_sink<S, A>(sink: S) -> ExtendSink<S>
where
    S: futures_sink::Sink<A>,
{
    ExtendSink::new(sink)
}
