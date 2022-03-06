use crate::{
    io::{self, ProcessUpdate},
    profiles::Profile,
};
use iced::{futures, Subscription};
use iced_native::subscription::Recipe;

pub fn stream(profile: Profile) -> iced::Subscription<io::ProcessUpdate> {
    Subscription::from_recipe(Process(profile))
}

struct Process(Profile);

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

        let mut cmd = Profile::start(&self.0);
        match io::stream_process(&mut cmd) {
            Ok(stream) => stream.boxed(),
            Err(err) => {
                let msg = err.to_string();
                futures::stream::once(async { ProcessUpdate::Error(msg) }).boxed()
            },
        }
    }
}
