use std::pin::Pin;

use futures::{
    task::{Context, Poll},
    Future, FutureExt,
};

/// Allows polling two futures untill only *one* completes (or bolth, if they complete at the same time)
///
/// Bolth the pending and the finished future are returned on completion of at least one of the futures
///
/// ## Note:
/// this implements Future, so you can `.await` it directly
pub struct OneOf<F0: Future, F1: Future> {
    fut0: Option<F0>,
    fut1: Option<F1>,
}

impl<F0: Future, F1: Future> OneOf<F0, F1> {
    /// Crates a new [`OneOf`] that polls the supplied future
    pub const fn new(f0: F0, f1: F1) -> Self {
        Self {
            fut0: Some(f0),
            fut1: Some(f1),
        }
    }
}

/// The result of a [`OneOf`] future completeing, containing ether one or bolth of the completed futures
pub enum OneOfResult<F0: Future, F1: Future> {
    F0(F0::Output, F1),
    F1(F0, F1::Output),
    /// This variant only occurs when bolth futures return [`Poll::Ready`] at the same time
    All(F0::Output, F1::Output),
}

impl<F0: Future + Unpin, F1: Future + Unpin> Future for OneOf<F0, F1> {
    type Output = OneOfResult<F0, F1>;
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        trace!("Poll");
        match (
            self.fut0.as_mut().unwrap().poll_unpin(cx),
            self.fut1.as_mut().unwrap().poll_unpin(cx),
        ) {
            (Poll::Pending, Poll::Pending) => Poll::Pending,
            (Poll::Ready(f0), Poll::Pending) => {
                trace!("f0 done");
                Poll::Ready(OneOfResult::F0(f0, self.fut1.take().unwrap()))
            }
            (Poll::Pending, Poll::Ready(f1)) => {
                trace!("f1 done");
                Poll::Ready(OneOfResult::F1(self.fut0.take().unwrap(), f1))
            }
            (Poll::Ready(f0), Poll::Ready(f1)) => Poll::Ready(OneOfResult::All(f0, f1)),
        }
    }
}
