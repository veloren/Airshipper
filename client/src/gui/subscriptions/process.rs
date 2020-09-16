use crate::io::{self, ProcessUpdate};
use iced::{futures, Subscription};
use iced_native::subscription::Recipe;
use tokio::process::Command;

pub fn stream(cmd: Command) -> iced::Subscription<io::ProcessUpdate> {
    Subscription::from_recipe(Process(cmd))
}

struct Process(Command);

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
        mut self: Box<Self>,
        _input: futures::stream::BoxStream<'static, I>,
    ) -> futures::stream::BoxStream<'static, Self::Output> {
        use iced::futures::stream::StreamExt;
        match io::stream_process(&mut self.0) {
            Ok(stream) => stream.boxed(),
            Err(err) => {
                futures::stream::once(async { ProcessUpdate::Error(err) }).boxed()
            },
        }
    }
}
