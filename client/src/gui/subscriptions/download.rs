use crate::net;
use iced::{futures, Subscription};
use std::path::{Path, PathBuf};

pub fn file(url: &str, location: &Path) -> iced::Subscription<net::Progress> {
    Subscription::from_recipe(Download(url.to_string(), location.to_path_buf()))
}

pub struct Download(String, PathBuf);

impl<H, I> iced_native::subscription::Recipe<H, I> for Download
where
    H: std::hash::Hasher,
{
    type Output = net::Progress;

    fn hash(&self, state: &mut H) {
        use std::hash::Hash;

        std::any::TypeId::of::<Self>().hash(state);
        self.0.hash(state);
    }

    fn stream(
        self: Box<Self>,
        _input: futures::stream::BoxStream<'static, I>,
    ) -> futures::stream::BoxStream<'static, Self::Output> {
        use iced::futures::stream::StreamExt;

        crate::net::download(self.0, self.1).boxed()
    }
}
