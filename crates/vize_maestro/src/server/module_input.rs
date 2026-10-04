//! Observe actual transport input termination before tower-lsp drains handlers.
use super::{ModuleLinkRetirement, ModuleLinkTerminationLease};
use futures::io::AsyncRead;
use std::{
    io::{self, IoSliceMut},
    pin::Pin,
    task::{Context, Poll},
};

pub(crate) struct ModuleLinkInput<I> {
    inner: I,
    termination: Option<ModuleLinkTerminationLease>,
}
impl<I> ModuleLinkInput<I> {
    pub(crate) fn new(inner: I, termination: Option<ModuleLinkTerminationLease>) -> Self {
        Self { inner, termination }
    }
    fn observe(&self, nonempty: bool, result: &Poll<io::Result<usize>>) {
        let reason = match result {
            Poll::Ready(Ok(0)) if nonempty => Some(ModuleLinkRetirement::InputEof),
            // AsyncRead must translate Interrupted/WouldBlock itself. Preserve
            // even a nonconforming reader's result without treating those as
            // terminal. Protocol/codec errors occur above this actual reader.
            Poll::Ready(Err(error))
                if !matches!(
                    error.kind(),
                    io::ErrorKind::Interrupted | io::ErrorKind::WouldBlock
                ) =>
            {
                Some(ModuleLinkRetirement::InputError)
            }
            _ => None,
        };
        if let (Some(reason), Some(termination)) = (reason, &self.termination) {
            termination.retire(reason);
        }
    }
}
impl<I: AsyncRead + Unpin> AsyncRead for ModuleLinkInput<I> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        let this = self.get_mut();
        let result = Pin::new(&mut this.inner).poll_read(cx, buf);
        this.observe(!buf.is_empty(), &result);
        result
    }
    fn poll_read_vectored(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        bufs: &mut [IoSliceMut<'_>],
    ) -> Poll<io::Result<usize>> {
        let this = self.get_mut();
        let nonempty = bufs.iter().any(|buf| !buf.is_empty());
        let result = Pin::new(&mut this.inner).poll_read_vectored(cx, bufs);
        this.observe(nonempty, &result);
        result
    }
}
#[cfg(test)]
mod tests;
