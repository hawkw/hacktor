use crate::{
    process::{self, Process},
    Message,
};
use futures::stream::FuturesUnordered;
use pin_utils;
use std::{
    future::Future,
    pin::Pin,
    task::{self, Poll},
};
use tokio_sync::{mpsc, oneshot};

pub struct Context<'a> {
    // used to wake the actor's task.
    task: &'a mut task::Context<'a>,
    // ...
}

/// A running process
pub struct Running<A, M>
where
    A: Process<M>,
    M: Message,
{
    /// The process's behavior.
    actor: A,

    /// Recieves incoming messages.
    inbox: mpsc::Receiver<Envelope<A, M>>,

    /// Configures the process's runtime characteristics.
    settings: process::Settings,

    /// Currently in flight response futures
    ///
    /// TODO(eliza): is it necessary for this to be ordered???
    processing: FuturesUnordered<Processing<A, M>>,
}

struct Responder<A, M>
where
    A: Process<M>,
    M: Message,
{
    tx: oneshot::Sender<Result<A::Response, HandleError<M, A::Error>>>,
}

pub struct HandleError<M, E> {
    kind: HandleErrorInner<M, E>,
}

enum HandleErrorInner<M, E> {
    InFlightLimit { max: usize, msg: M },
    Disconnected(M),
    Inner(E),
}

/// In flight messages
struct Envelope<A, M>
where
    A: Process<M>,
    M: Message,
{
    msg: M,
    tx: Responder<A, M>,
}

struct Processing<A, M>
where
    A: Process<M>,
    M: Message,
{
    future: A::Future,
    tx: Option<Responder<A, M>>,
}

// === impl Context ===

impl<'a> Context<'a> {
    /// Returns a reference to the `Waker` for the current task.
    #[inline]
    pub fn waker(&self) -> &'a task::Waker {
        &self.task.waker()
    }
}

// === impl Running ===

impl<A, M> Running<A, M>
where
    A: Process<M>,
    M: Message,
{
    fn handle<'cx>(
        &mut self,
        Envelope { msg, tx }: Envelope<A, M>,
        task: &'cx mut task::Context<'cx>,
    ) {
        if self.processing.len() >= self.settings.max_in_flight {
            tx.max_in_flight(msg, self.settings.max_in_flight);
        } else {
            let mut cx = Context { task };
            let future = self.actor.recv(msg, &mut cx);
            self.processing.push(Processing {
                future,
                tx: Some(tx),
            });
        }
    }
}

impl<A, M> Future for Running<A, M>
where
    A: Process<M>,
    M: Message,
{
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> Poll<Self::Output> {
        unimplemented!()
    }
}

// === impl Processing ===

impl<A, M> Processing<A, M>
where
    A: Process<M>,
    M: Message,
{
    pin_utils::unsafe_pinned!(future: A::Future);
    pin_utils::unsafe_unpinned!(tx: Option<Responder<A, M>>);
}

impl<A, M> Future for Processing<A, M>
where
    A: Process<M>,
    M: Message,
{
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> Poll<Self::Output> {
        // First, make sure the receiver still exists before driving the inner
        // future.
        if let Poll::Ready(()) = self
            // can't believe we have to do this horrible dance because of stack
            // pinning, whatever... x_x
            .as_mut()
            .tx()
            .as_mut()
            .expect("polled after ready")
            .poll_closed(cx)
        {
            tracing::trace!(message = "terminating", reason = "tx closed");
            return Poll::Ready(());
        }

        // Next, try to drive the inner future...
        let res = futures::ready!(self.as_mut().future().as_mut().poll(cx));

        // If the future has completed, send it back to the receiver.
        let tx = self.tx(); // (weird pin thing)
        tx.take().expect("polled after ready").respond(res);
        Poll::Ready(())
    }
}

// === impl Responder ===

impl<A, M> Responder<A, M>
where
    A: Process<M>,
    M: Message,
{
    fn poll_closed(&mut self, cx: &mut task::Context<'_>) -> Poll<()> {
        self.tx.poll_closed(cx)
    }

    fn respond(self, item: Result<A::Response, A::Error>) {
        if let Err(_) = self.tx.send(item.map_err(HandleError::inner)) {
            tracing::trace!("rx dropped");
        }
    }

    fn max_in_flight(self, msg: M, max: usize) {
        if let Err(_) = self.tx.send(Err(HandleError::max_in_flight(msg, max))) {
            tracing::trace!("rx dropped");
        }
    }
}

// === impl HandleError ===

impl<M, E> HandleError<M, E> {
    fn inner(inner: E) -> Self {
        Self {
            kind: HandleErrorInner::Inner(inner),
        }
    }

    fn max_in_flight(msg: M, max: usize) -> Self {
        Self {
            kind: HandleErrorInner::InFlightLimit { max, msg },
        }
    }
}
