use crate::{rt, Message};
use std::{
    future::Future,
    pin::Pin,
    // marker::Unpin,
    task::{self, Poll},
};

/// This is the actor trait, but I wasn't sure if it ought to be called that.
pub trait Process<M: Message> {
    type Response: Message;
    type Error: Message; // TODO: + std::error::Error?
    type Future: Future<Output = Result<Self::Response, Self::Error>>;

    /// Returns `Ready` once this actor is ready to receive a message.
    ///
    /// Otherwise, backpressure will be exerted.
    fn poll_ready(&mut self, cx: &mut rt::Context<'_>) -> Poll<Result<(), Self::Error>>;

    /// Handles a message, returning a future.
    fn recv(&mut self, message: M, cx: &mut rt::Context<'_>) -> Self::Future;
}

/// A reference to a local or remote actor.
pub trait Ref<A> {
    fn send<M>(&mut self, msg: M) -> SendFuture<A, M>
    where
        A: Process<M>,
        M: Message;

    fn poll_ready<M>(&mut self) -> Poll<Result<(), A::Error>>
    where
        A: Process<M>,
        M: Message;
}

/// Configuration for a process
#[derive(Clone, Debug)]
pub struct Settings {
    pub(crate) max_in_flight: usize,
    pub(crate) inbox_size: usize,
}

pub struct SendFuture<A, M> {
    // TODO
    _p: std::marker::PhantomData<fn(A, M)>,
}

impl<A, M> Future for SendFuture<A, M>
where
    A: Process<M>,
    M: Message,
{
    type Output = Result<A::Response, A::Error>;
    fn poll(self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> Poll<Self::Output> {
        unimplemented!()
    }
}
