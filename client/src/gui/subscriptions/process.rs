use crate::{
    io::{self, ProcessUpdate},
    CommandBuilder,
};
use iced::{futures, Subscription};
use iced_native::subscription::Recipe;

pub fn stream(cmd: &CommandBuilder) -> iced::Subscription<io::ProcessUpdate> {
    Subscription::from_recipe(Process(cmd.clone()))
}

pub(crate) struct Process(CommandBuilder);

impl<H, I> Recipe<H, I> for Process
where
    H: std::hash::Hasher,
{
    type Output = ProcessUpdate;

    fn hash(&self, state: &mut H) {
        use std::hash::Hash;

        std::any::TypeId::of::<Self>().hash(state);
        // TODO: is exploiting the Debug impl for hashing a good idea?
        format!("{:?}", self.0).hash(state);
    }

    fn stream(
        self: Box<Self>,
        _input: futures::stream::BoxStream<'static, I>,
    ) -> futures::stream::BoxStream<'static, Self::Output> {
        use iced::futures::stream::StreamExt;

        crate::io::stream_process(self.0).boxed()
    }
}
