use crate::{
    io::{self, ProcessUpdate},
    profiles::Profile,
};
use iced::{futures, Subscription};
use iced_native::subscription::Recipe;

pub fn stream(
    profile: Profile,
    game_server_address: Option<String>,
) -> iced::Subscription<io::ProcessUpdate> {
    Subscription::from_recipe(Process {
        profile,
        game_server_address,
    })
}

struct Process {
    profile: Profile,
    game_server_address: Option<String>,
}

impl<H, I> Recipe<H, I> for Process
where
    H: std::hash::Hasher,
{
    type Output = ProcessUpdate;

    fn hash(&self, state: &mut H) {
        use std::hash::Hash;

        std::any::TypeId::of::<Self>().hash(state);
        // TODO: is exploiting the Debug impl for hashing a good idea?
        format!("{:?}", self.profile).hash(state);
    }

    fn stream(
        self: Box<Self>,
        _input: futures::stream::BoxStream<'static, I>,
    ) -> futures::stream::BoxStream<'static, Self::Output> {
        use iced::futures::stream::StreamExt;

        let mut cmd = Profile::start(&self.profile, self.game_server_address.as_deref());
        match io::stream_process(&mut cmd) {
            Ok(stream) => stream.boxed(),
            Err(err) => {
                let msg = err.to_string();
                futures::stream::once(async { ProcessUpdate::Error(msg) }).boxed()
            },
        }
    }
}
