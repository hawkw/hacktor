use std::{
    future::Future,
    // marker::Unpin,
    task::{self, Poll},
    pin::Pin,
};

use serde::{Serialize, Deserialize};

pub struct Context<'a> {
    // used to wake the actor's task.
    task: task::Context<'a>,
    // ...
}

/// A message that can be sent between threads, over IPC, or across the network.
pub trait Message: Serialize + for<'de> Deserialize<'de> + Send {}

/// This is the actor trait, but I wasn't sure if it ought to be called that.
pub trait Process<M: Message> {
    type Response: Message;
    type Error: Message; // TODO: + std::error::Error?
    type Future: Future<Output = Result<Self::Response, Self::Error>>;

    /// Returns `Ready` once this actor is ready to receive a message.
    ///
    /// Otherwise, backpressure will be exerted.
    fn poll_ready(&mut self, ctx: &mut Context<'_>) -> Poll<Result<(), Self::Error>>;

    /// Handles a message, returning a future.
    fn handle(&mut self, message: M, ctx: &mut Context<'_>) -> Self::Future;
}

/// A reference to a local or remote actor.
pub trait Ref<A> {
    fn send<M>(&mut self, msg: M) -> SendFuture<<A as Process<M>>::Response, <A as Process<M>::Error>>
    where
        A: Process<M>,
        M: Message;
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

impl<M> Message for M where M: Serialize + for<'de> Deserialize<'de> + Send {}
