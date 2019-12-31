use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;

use indicatif::{ProgressBar, ProgressStyle};
use reqwest::header;
use reqwest::Url;

use crate::Result;

struct DownloadBar<R> {
    inner: R,
    progress_bar: ProgressBar,
}

impl<R: Read> Read for DownloadBar<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.inner.read(buf).map(|n| {
            self.progress_bar.inc(n as u64);
            n
        })
    }
}

/// Downloads a file with a progress bar
pub fn download_with_progress(
    client: &crate::network::CLIENT,
    url: &str,
    file: &PathBuf,
) -> Result<fs::File> {
    let url = match Url::parse(url) {
        Ok(x) => x,
        Err(e) => return Err(format!("failed to parse Url: {}", e).into()),
    };

    let total_size = {
        let resp = client.head(url.as_str()).send()?;
        if resp.status().is_success() {
            resp.headers()
                .get(header::CONTENT_LENGTH)
                .and_then(|ct_len| ct_len.to_str().ok())
                .and_then(|ct_len| ct_len.parse().ok())
                .unwrap_or(0)
        } else {
            return Err(format!(
                "Couldn't download from '{}' because server returned {:?}",
                url,
                resp.status(),
            )
            .into());
        }
    };

    let mut request = client.get(url.as_str());
    let pb = ProgressBar::new(total_size);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] [{bar:40.green/white}] {bytes}/{total_bytes} ({eta})")
            .progress_chars("=>-"),
    );

    if file.exists() {
        let size = file.metadata()?.len() - 1;
        request = request.header(header::RANGE, format!("bytes={}-", size));
        pb.inc(size);
    }

    let mut source = DownloadBar {
        progress_bar: pb,
        inner: request.send()?,
    };

    let mut dest = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .read(true)
        .append(false)
        .truncate(true)
        .open(&file)?;

    let _ = io::copy(&mut source, &mut dest)?;

    log::info!("Download completed.");
    Ok(dest)
}

pub fn unzip_with_progress(mut zip_file: fs::File, destination: &PathBuf) -> Result<()> {
    use std::convert::TryInto;

    let mut archive = zip::ZipArchive::new(&mut zip_file)?;
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_chars("|/-\\")
            .template("{spinner} {wide_msg}"),
    );
    pb.set_length(
        archive
            .len()
            .try_into()
            .expect("Archive file too big to comprehend :/"),
    );
    pb.enable_steady_tick(1000);

    for i in 1..archive.len() {
        let mut file = archive.by_index(i)?;
        let path = destination.join(file.sanitized_name());

        if file.is_dir() {
            std::fs::create_dir_all(path)?;
        } else {
            pb.set_message("Unzipping...");
            let mut target = fs::OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(path)?;

            std::io::copy(&mut file, &mut target)?;
        }
    }
    pb.finish_with_message("Unzipped.");

    Ok(())
}
