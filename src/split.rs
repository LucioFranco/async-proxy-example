use std::io;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio_io::{AsyncRead, AsyncWrite};
use tokio_sync::lock::Lock;

macro_rules! ready {
    ($e:expr) => {
        match $e {
            ::std::task::Poll::Ready(t) => t,
            ::std::task::Poll::Pending => return ::std::task::Poll::Pending,
        }
    };
}

pub fn split<T>(t: T) -> (ReadHalf<T>, WriteHalf<T>)
where
    T: AsyncRead + AsyncWrite + Unpin,
{
    let a = Lock::new(t);
    let b = a.clone();
    (ReadHalf { handle: a }, WriteHalf { handle: b })
}

pub struct WriteHalf<T> {
    handle: Lock<T>,
}

pub struct ReadHalf<T> {
    handle: Lock<T>,
}

impl<T: Unpin> Unpin for ReadHalf<T> {}

impl<T> AsyncRead for ReadHalf<T>
where
    T: AsyncRead + Unpin,
{
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        let mut handle = ready!(self.handle.poll_lock(cx));
        Pin::new(&mut *handle).poll_read(cx, buf)
    }
}

impl<T: Unpin> Unpin for WriteHalf<T> {}

impl<T> AsyncWrite for WriteHalf<T>
where
    T: AsyncWrite + Unpin,
{
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        let mut handle = ready!(self.handle.poll_lock(cx));
        Pin::new(&mut *handle).poll_write(cx, buf)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        let mut handle = ready!(self.handle.poll_lock(cx));
        Pin::new(&mut *handle).poll_flush(cx)
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        let mut handle = ready!(self.handle.poll_lock(cx));
        Pin::new(&mut *handle).poll_shutdown(cx)
    }
}
