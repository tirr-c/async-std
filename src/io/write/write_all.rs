use crate::future::Future;
use crate::task::{Context, Poll};

use std::io;
use std::mem;
use std::pin::Pin;

use futures_io::AsyncWrite;

#[doc(hidden)]
#[allow(missing_debug_implementations)]
pub struct WriteAllFuture<'a, T: Unpin + ?Sized> {
    pub(crate) writer: &'a mut T,
    pub(crate) buf: &'a [u8],
}

impl<T: AsyncWrite + Unpin + ?Sized> Future for WriteAllFuture<'_, T> {
    type Output = io::Result<()>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let Self { writer, buf } = &mut *self;

        while !buf.is_empty() {
            let n = futures_core::ready!(Pin::new(&mut **writer).poll_write(cx, buf))?;
            let (_, rest) = mem::replace(buf, &[]).split_at(n);
            *buf = rest;

            if n == 0 {
                return Poll::Ready(Err(io::ErrorKind::WriteZero.into()));
            }
        }

        Poll::Ready(Ok(()))
    }
}
