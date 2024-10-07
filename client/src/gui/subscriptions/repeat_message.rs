use std::{hash::Hash, time::Duration};

use futures_util::stream::BoxStream;
use iced::{futures, subscription::Recipe};

pub fn stream<M>(interval: Duration, message: M) -> iced::Subscription<M>
where
    M: Clone + Send + Sync + 'static,
{
    iced::Subscription::from_recipe(RepeatMessageStream(interval, message))
}

struct RepeatMessageStream<M>(Duration, M);

impl<H, E, M> Recipe<H, E> for RepeatMessageStream<M>
where
    H: core::hash::Hasher,
    M: Clone + Send + Sync + 'static,
{
    type Output = M;

    fn hash(&self, state: &mut H) {
        core::any::TypeId::of::<RepeatMessageStream<M>>().hash(state);
        self.0.hash(state);
    }

    fn stream(self: Box<Self>, _input: BoxStream<E>) -> BoxStream<Self::Output> {
        Box::pin(futures::stream::unfold(
            (tokio::time::interval(self.0), self.1.clone()),
            |(mut interval, message)| async move {
                interval.tick().await;
                Some((message.clone(), (interval, message)))
            },
        ))
    }
}
