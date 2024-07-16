//! e.g. run cargo run -- ~/.local/share/airshipper/profiles/default https://github.com/veloren/veloren/releases/download/nightly/nightly-linux-x86_64-2024-06-09T16_04.zip
use std::path::{Path, PathBuf};

use clap::Parser;
use compare::CompareEntry;
use local_directory::LocalDirectory;
use remote_zip::RemoteZip;
use tokio::runtime::Runtime;

mod compare;
mod local_directory;
mod remote_zip;
mod util;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// specify which directory to update with the fast update method
    veloren_airshipper_dir: PathBuf,

    /// specify source to update local dir from
    online_zip: String,
}

fn main() {
    let cli = Cli::parse();
    let runtime = Runtime::new().unwrap();

    let mut localdir = LocalDirectory::new(cli.veloren_airshipper_dir.clone());

    let client = reqwest::Client::builder()
        .http2_prior_knowledge()
        .build()
        .unwrap();
    let url = reqwest::Url::parse(&cli.online_zip).unwrap();
    let mut zip = RemoteZip::new(client, url);

    let (local_files, remote_files) = runtime.block_on(async {
        tokio::join!(
            localdir.fetch_file_information(),
            zip.fetch_file_information()
        )
    });

    let local_files = local_files.unwrap();
    let remote_files = remote_files.unwrap();

    let comparison = compare::compare_local_with_remote(remote_files, local_files);
    let mut data_equal = 0;
    let mut data_changed = 0;
    let mut changed_files = Vec::new();
    let mut new_files = Vec::new();
    for e in comparison.iter() {
        match e {
            CompareEntry::EqualCrc(remote, _) => {
                data_equal += remote.fixed.compressed_size;
            },
            CompareEntry::DifferentCrc(remote, local) => {
                data_changed += remote.fixed.compressed_size;
                changed_files.push(local.local_path.clone());
            },
            CompareEntry::ExistsInRemote(remote) => {
                data_changed += remote.fixed.compressed_size;
                new_files.push(std::str::from_utf8(&remote.file_name).unwrap());
            },
            _ => (),
        }
    }
    let supported_compression = util::compression_method_supported(&comparison);

    println!("Analysation complete:");
    println!("need to download: {} MBs", data_changed / 1000000);
    println!("could skip downloading: {} MBs", data_equal / 1000000);
    println!("Support remote compression: {}", supported_compression);

    println!("changed files:");
    for f in changed_files {
        println!("changed: {f}");
    }
    println!("new files:");
    for f in new_files {
        println!("new: {f}");
    }

    println!("patching files:");
    runtime.block_on(download_all(
        &cli.veloren_airshipper_dir,
        &mut zip,
        comparison,
    ));
    println!("files patched");
}

async fn download_all(
    basedir: &Path,
    zip: &mut RemoteZip,
    comparison: Vec<CompareEntry>,
) {
    use tokio::{fs::File, io::AsyncWriteExt}; // for write_all()

    let mut i = 0;
    for e in comparison.iter() {
        match e {
            CompareEntry::DifferentCrc(remote, _) => {
                let mut remote_data = zip.download(remote).await.unwrap();
                let filename: PathBuf =
                    std::str::from_utf8(&remote.file_name).unwrap().into();
                let mut file_path = basedir.to_path_buf();
                file_path.push(filename);

                let mut file = File::create(file_path).await.unwrap();
                println!("{i}: {:?}", std::str::from_utf8(&remote.file_name));
                i += 1;
                file.write_all_buf(&mut remote_data).await.unwrap();
            },
            CompareEntry::ExistsInRemote(remote) => {
                let mut remote_data = zip.download(remote).await.unwrap();
                let filename: PathBuf =
                    std::str::from_utf8(&remote.file_name).unwrap().into();
                let mut file_path = basedir.to_path_buf();
                file_path.push(filename);

                let mut file = File::create(file_path).await.unwrap();
                println!("{i}: {:?}", std::str::from_utf8(&remote.file_name));
                i += 1;
                file.write_all_buf(&mut remote_data).await.unwrap();
            },
            CompareEntry::ExistsInLocal(local) => {
                tokio::fs::remove_file(&local.path).await.unwrap();
            },
            _ => (),
        }
    }
}
