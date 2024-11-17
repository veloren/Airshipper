use std::{hash::Hash, time::Duration};

use iced::{
    advanced::{
        subscription::{EventStream, Recipe},
        Hasher,
    },
    futures::{self, stream::BoxStream},
    Subscription,
};

pub fn stream<M>(interval: Duration, message: M) -> iced::Subscription<M>
where
    M: Clone + Send + Sync + 'static,
{
    Subscription::from_recipe(RepeatMessageStream(interval, message))
}

struct RepeatMessageStream<M>(Duration, M);

impl<M> Recipe for RepeatMessageStream<M>
where
    M: Clone + Send + Sync + 'static,
{
    type Output = M;

    fn hash(&self, state: &mut Hasher) {
        core::any::TypeId::of::<RepeatMessageStream<M>>().hash(state);
        self.0.hash(state);
    }

    fn stream(self: Box<Self>, _input: EventStream) -> BoxStream<'static, Self::Output> {
        Box::pin(futures::stream::unfold(
            (tokio::time::interval(self.0), self.1.clone()),
            |(mut interval, message)| async move {
                interval.tick().await;
                Some((message.clone(), (interval, message)))
            },
        ))
    }
}
