use super::{IntoStream, Stream};

use std::pin::Pin;

/// Conversion from a `Stream`.
///
/// By implementing `FromStream` for a type, you define how it will be created from a stream.
/// This is common for types which describe a collection of some kind.
///
/// See also: [`IntoStream`].
///
/// [`IntoStream`]: trait.IntoStream.html
#[cfg_attr(feature = "docs", doc(cfg(unstable)))]
#[cfg(any(feature = "unstable", feature = "docs"))]
pub trait FromStream<T: Send> {
    /// Creates a value from a stream.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// // use async_std::stream::FromStream;
    ///
    /// // let _five_fives = async_std::stream::repeat(5).take(5);
    /// ```
    fn from_stream<'a, S: IntoStream<Item = T> + Send + 'a>(
        stream: S,
    ) -> Pin<Box<dyn core::future::Future<Output = Self> + Send + 'a>>;
}

impl FromStream<()> for () {
    fn from_stream<'a, S: IntoStream<Item = ()> + Send + 'a>(
        stream: S,
    ) -> Pin<Box<dyn core::future::Future<Output = Self> + Send + 'a>>
    {
        let stream = stream.into_stream();
        Box::pin(async move {
            pin_utils::pin_mut!(stream);
            while let Some(_) = stream.next().await {}
        })
    }
}
