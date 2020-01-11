use {
    async_std::stream::interval,
    futures::stream::{BoxStream, StreamExt},
    iced_native::subscription::Recipe,
    std::{hash::Hash, time::Duration},
};

pub fn every(duration: std::time::Duration) -> iced::Subscription<()> {
    iced::Subscription::from_recipe(Every(duration))
}

struct Every(Duration);

impl<H, I> Recipe<H, I> for Every
where
    H: std::hash::Hasher,
{
    type Output = ();

    fn hash(&self, state: &mut H) {
        std::any::TypeId::of::<Self>().hash(state);
        self.0.hash(state);
    }

    fn stream(self: Box<Self>, _input: I) -> BoxStream<'static, Self::Output> {
        interval(self.0).boxed()
    }
}
