use iced::{futures, subscription, Subscription};
use std::path::{Path, PathBuf};

use crate::{
    update::{Progress, Storage},
    GITHUB_CLIENT,
};

pub fn file(url: &str, download_path: &Path) -> iced::Subscription<Progress> {
    Subscription::from_recipe(Download(url.to_string(), download_path.to_path_buf()))
}

pub struct Download(String, PathBuf);

impl<H, I> subscription::Recipe<H, I> for Download
where
    H: std::hash::Hasher,
{
    type Output = crate::update::Progress;

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

        let rb = GITHUB_CLIENT.get(self.0);
        let storage = Storage::FileInfo(self.1);
        crate::update::download_stream(rb, storage, ()).boxed()
    }
}
