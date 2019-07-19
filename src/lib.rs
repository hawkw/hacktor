use std::{
    future::Future,
    // marker::Unpin,
    task::{self, Poll},
};

pub struct Context<'a> {
    // used to wake the actor's task.
    task: task::Context<'a>,
    // ...
}

/// This is the actor trait, but I wasn't sure if it ought to be called that.
pub trait Process<M> {
    type Yield;
    type Error;
    type Future: Future<Output = Result<Self::Yield, Self::Error>>;

    /// Returns `Ready` once this actor is ready to receive a message.
    ///
    /// Otherwise, backpressure will be exerted.
    fn poll_ready(&mut self, ctx: &mut Context<'_>) -> Poll<Result<(), Self::Error>>;

    /// Handles a message, returning a future.
    fn handle(&mut self, message: M, ctx: &mut Context<'_>) -> Self::Future;
}

/// A reference to a local or remote actor.
pub trait Ref<A> {
    fn send<M>(&mut self, msg: M) -> SendFuture<<A as Process<M>>::Yield, <A as Process<M>::Error>>
    where
        A: Process<M>;
}

pub struct SendFuture<I, E> {
    // TODO
    _p: std::marker::PhantomData<(I, E)>,
}
