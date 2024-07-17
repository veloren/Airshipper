use std::{convert::TryFrom, io::Read, time::Duration};

use crate::{
    consts::DOWNLOAD_FILE, error::ClientError, profiles::Profile, GITHUB_CLIENT,
    WEB_CLIENT,
};
use afterburner::Afterburner;
use bytes::{Buf, BytesMut};
use compare::{prepare_local_with_remote, Compared};
use download::{Download, DownloadContent, InternalProgressData, ProgressData, Storage};
use flate2::read::DeflateDecoder;
use futures_util::stream::Stream;
use iced::futures;
use local_directory::{FileInformation, LocalDirectory};
use remote_zip::{gen_classsic, RemoteZipError};
use tokio::io::AsyncWriteExt;
use zip_core::{
    raw::{parse::Parse, CentralDirectoryHeader, LocalFileHeader},
    structs::CompressionMethod,
    Signature,
};

mod afterburner;
mod compare;
mod download;
mod local_directory;
mod remote_zip;

#[derive(Debug, thiserror::Error)]
pub(crate) enum UpdateError {
    #[error("Failed to access Airshippers Server for download information: {0}")]
    FailedToAccessAirshipperServer(ClientError),
    #[error("Failed to download the Veloren update, probably from github: {0}")]
    FailedToAccessVelorenDownload(ClientError),
    #[error("Custom Error: {0}")]
    Custom(String),
}

#[derive(Debug)]
pub enum Progress {
    Evaluating,
    /// If the consumer sees ReadyToDownload a download is necessary, but they can
    /// implement logic to avoid any download
    ReadyToDownload,
    #[allow(clippy::enum_variant_names)]
    InProgress(ProgressData),
    Sucessful,
    Errored(UpdateError),
}

#[derive(Debug, Clone)]
pub struct UpdateParameters {
    pub profile: Profile,
    /// skip all checks and directly start downloading
    pub force_complete_redownload: bool,
}

#[derive(Debug)]
enum State {
    ToBeEvaluated(UpdateParameters),
    DownloadEndOfCentralDirectory(UpdateParameters, bool),
    DownloadCentralDirectory(UpdateParameters, bool, Download<()>),
    CrcLocalDirectory(
        UpdateParameters,
        LocalDirectory,
        Vec<CentralDirectoryHeader>,
    ),
    DownloadingClassic(UpdateParameters, Download<()>),
    UnzipClassic(UpdateParameters),
    FilterMissingFiles(
        UpdateParameters,
        Vec<CentralDirectoryHeader>,
        Vec<FileInformation>,
    ),
    DownloadingPartially(
        UpdateParameters,
        Compared,
        Download<CentralDirectoryHeader>,
        Vec<(BytesMut, CentralDirectoryHeader, LocalFileHeader)>,
        Afterburner<CentralDirectoryHeader>,
        InternalProgressData,
    ),
    UnzipPartial(
        UpdateParameters,
        Compared,
        Vec<(BytesMut, CentralDirectoryHeader, LocalFileHeader)>,
        InternalProgressData,
    ),
    RemovingPartially(UpdateParameters, Compared),
    Finished,
}

pub(crate) fn update(params: UpdateParameters) -> impl Stream<Item = Progress> {
    tracing::debug!(?params, "start updating");
    futures::stream::unfold(State::ToBeEvaluated(params), |old_state| async move {
        let res = match old_state {
            State::ToBeEvaluated(params) => evalute_remote_version(params).await,
            State::DownloadEndOfCentralDirectory(p, m) => {
                evaluate_remote_eocd(p, m).await
            },
            State::DownloadCentralDirectory(p, m, d) => evaluate_remote_cd(p, m, d).await,
            State::CrcLocalDirectory(p, ld, cds) => evaluate_local_dir(p, ld, cds).await,
            State::DownloadingClassic(p, d) => downloading_classic(p, d).await,
            State::UnzipClassic(p) => unzip_classic(p).await,
            State::FilterMissingFiles(p, cds, fi) => filter_missings(p, cds, fi).await,
            State::DownloadingPartially(p, cp, d, f, a, pr) => {
                downloading_partial(p, cp, d, f, a, pr).await
            },
            State::UnzipPartial(p, cp, f, pr) => unzip_partial(p, cp, f, pr).await,
            State::RemovingPartially(p, cp) => removing_partial(p, cp).await,
            State::Finished => Ok(None),
        };
        match res {
            Ok(ok) => ok,
            Err(e) => Some((Progress::Errored(e), State::Finished)),
        }
    })
}

// asks airshipper server for version and compares with profile
async fn evalute_remote_version(
    params: UpdateParameters,
) -> Result<Option<(Progress, State)>, UpdateError> {
    tracing::debug!("evalute_remote_version");
    if params.force_complete_redownload {
        return Ok(Some((Progress::ReadyToDownload, gen_classsic(params))));
    }

    let map_err =
        |e: reqwest::Error| UpdateError::FailedToAccessAirshipperServer(e.into());
    let remote = WEB_CLIENT
        .get(params.profile.version_url())
        .send()
        .await
        .map_err(map_err)?
        .text()
        .await
        .map_err(map_err)?;

    let remote_matches_profile =
        Some(remote) != params.profile.version || !&params.profile.installed();

    Ok(Some((
        Progress::Evaluating,
        State::DownloadEndOfCentralDirectory(params, remote_matches_profile),
    )))
}

fn fallback(
    params: &UpdateParameters,
    remote_matches_profile: bool,
) -> Result<Option<(Progress, State)>, UpdateError> {
    if remote_matches_profile {
        // If content_length is unavailable and profile matches, we assume
        // everything is fine
        Ok(Some((Progress::Sucessful, State::Finished)))
    } else {
        // if profiles dont match, we enforce classic download
        Ok(Some((
            Progress::ReadyToDownload,
            gen_classsic(params.clone()),
        )))
    }
}

// does 1 HEAD and 1 gets to get zip internal information
async fn evaluate_remote_eocd(
    params: UpdateParameters,
    remote_matches_profile: bool,
) -> Result<Option<(Progress, State)>, UpdateError> {
    tracing::debug!("evalute_remote_cd");
    let zip_url = params.profile.download_url();
    let map_err =
        |e: reqwest::Error| UpdateError::FailedToAccessVelorenDownload(e.into());
    let content_length_resp =
        GITHUB_CLIENT.head(&zip_url).send().await.map_err(map_err)?;

    let content_length = match content_length_resp.content_length() {
        Some(len) => len,
        None => return fallback(&params, remote_matches_profile),
    };

    // download EOCD
    let eocd = match remote_zip::download_eocd(content_length, &zip_url).await {
        Ok(eocd) => eocd,
        Err(RemoteZipError::Reqwest(e)) => {
            return Err(UpdateError::FailedToAccessVelorenDownload(e.into()));
        },
        Err(_) => return fallback(&params, remote_matches_profile),
    };

    tracing::trace!("starting CD download");
    let download = remote_zip::download_cds(&eocd, &zip_url);

    Ok(Some((
        Progress::Evaluating,
        State::DownloadCentralDirectory(params, remote_matches_profile, download),
    )))
}

// fetching the CD might take ~1,5s, so do it partially
async fn evaluate_remote_cd(
    params: UpdateParameters,
    remote_matches_profile: bool,
    download: Download<()>,
) -> Result<Option<(Progress, State)>, UpdateError> {
    let download = download
        .progress()
        .await
        .map_err(|e| UpdateError::Custom(format!("{:?}", e)))?;
    match download {
        Download::Finished(storage, ()) => {
            //download CDs
            let bytes = match storage {
                Storage::Memory(bytes) => bytes.freeze(),
                _ => unreachable!(),
            };
            let cds = match remote_zip::extract_cds(bytes) {
                Some(cds) => cds,
                None => {
                    return fallback(&params, remote_matches_profile);
                },
            };

            tracing::trace!("starting verifying disk");
            let local_directory = LocalDirectory::Start(params.profile.directory());

            Ok(Some((
                Progress::Evaluating,
                State::CrcLocalDirectory(params, local_directory, cds),
            )))
        },
        Download::Progress(_, _, _, _) => Ok(Some((
            Progress::Evaluating,
            State::DownloadCentralDirectory(params, remote_matches_profile, download),
        ))),
        Download::Start(_, _, _, _) => unreachable!(),
    }
}

// crc all local files to check for diffs
async fn evaluate_local_dir(
    params: UpdateParameters,
    local_dir: LocalDirectory,
    cds: Vec<CentralDirectoryHeader>,
) -> Result<Option<(Progress, State)>, UpdateError> {
    let local_dir = local_dir
        .progress()
        .await
        .map_err(|e| UpdateError::Custom(format!("{:?}", e)))?;
    match local_dir {
        LocalDirectory::Finished(file_infos) => Ok(Some((
            Progress::Evaluating,
            State::FilterMissingFiles(params, cds, file_infos),
        ))),
        LocalDirectory::Progress(_, _, _, _, _) => Ok(Some((
            Progress::Evaluating,
            State::CrcLocalDirectory(params, local_dir, cds),
        ))),
        LocalDirectory::Start(_) => unreachable!(),
    }
}

// continues_downloading
async fn downloading_classic(
    params: UpdateParameters,
    download: Download<()>,
) -> Result<Option<(Progress, State)>, UpdateError> {
    let download = download
        .progress()
        .await
        .map_err(|e| UpdateError::Custom(format!("{:?}", e)))?;
    match &download {
        Download::Finished(_, ()) => Ok(Some((
            Progress::InProgress(ProgressData::new(0, DownloadContent::FullZip)),
            State::UnzipClassic(params),
        ))),
        Download::Progress(_, _, progress, _) => Ok(Some((
            Progress::InProgress(progress.progress.clone()),
            State::DownloadingClassic(params, download),
        ))),
        Download::Start(_, _, _, _) => unreachable!(),
    }
}

// continues_downloading
async fn unzip_classic(
    params: UpdateParameters,
) -> Result<Option<(Progress, State)>, UpdateError> {
    let dir = params.profile.directory();
    tracing::info!("Unzipping to {:?}", &dir);
    let mut zip_file = std::fs::File::open(dir.join(DOWNLOAD_FILE))
        .map_err(|e| UpdateError::Custom(format!("{:?}", e)))?;

    let mut archive = zip::ZipArchive::new(&mut zip_file)
        .map_err(|e| UpdateError::Custom(format!("{:?}", e)))?;

    // Delete all assets to ensure that no obsolete assets will remain.
    let assets = params.profile.directory().join("assets");
    if assets.exists() {
        std::fs::remove_dir_all(assets)
            .map_err(|e| UpdateError::Custom(format!("{:?}", e)))?;
    }

    for i in 1..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| UpdateError::Custom(format!("{:?}", e)))?;
        // TODO: Verify that `sanitized_name()` works correctly in this case.
        #[allow(deprecated)]
        let path = params.profile.directory().join(file.sanitized_name());

        if file.is_dir() {
            std::fs::create_dir_all(path)
                .map_err(|e| UpdateError::Custom(format!("{:?}", e)))?;
        } else {
            let mut target = std::fs::OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(path)
                .map_err(|e| UpdateError::Custom(format!("{:?}", e)))?;

            std::io::copy(&mut file, &mut target)
                .map_err(|e| UpdateError::Custom(format!("{:?}", e)))?;
        }
    }

    Ok(Some((Progress::Sucessful, State::Finished)))
}

// compares if files are different and prepares next steps
async fn filter_missings(
    params: UpdateParameters,
    cds: Vec<CentralDirectoryHeader>,
    file_infos: Vec<FileInformation>,
) -> Result<Option<(Progress, State)>, UpdateError> {
    tracing::debug!("filter_missings");
    let mut compared = prepare_local_with_remote(cds, file_infos);

    tracing::debug!("need to download {} files", compared.needs_redownload.len());
    tracing::debug!("need to delete {} files", compared.needs_deletion_total);
    tracing::trace!("{} bytes clean", compared.clean_data_total);

    use std::convert::TryFrom;
    if compared.needs_redownload.iter().any(|cd| {
        !matches!(
            CompressionMethod::try_from(cd.fixed.compression_method),
            Ok(CompressionMethod::Deflated) | Ok(CompressionMethod::Stored)
        )
    }) {
        tracing::warn!(
            "unsupported compression method found, falling back to classic mode"
        );
        return fallback(&params, false);
    }

    const MAX_FILES_FOR_PARTIAL_DOWNLOAD: usize = 100;
    if compared.needs_redownload.len() > MAX_FILES_FOR_PARTIAL_DOWNLOAD {
        tracing::info!(
            "to many files changed to make partial download efficient, falling back to \
             classic mode"
        );
        return fallback(&params, false);
    }

    match remote_zip::next_partial(&params, &mut compared) {
        Some(first_partially) => {
            let content = match &first_partially {
                Download::Start(_, _, c, _) => c,
                _ => unreachable!(),
            };

            let progress = ProgressData {
                bytes_per_sec: 0,
                downloaded_bytes: 0,
                total_bytes: compared.needs_redownload_bytes,
                content: content.clone(),
            };
            let iprogress = InternalProgressData::new(progress);

            Ok(Some((
                Progress::ReadyToDownload,
                State::DownloadingPartially(
                    params,
                    compared,
                    first_partially,
                    vec![],
                    Afterburner::default(),
                    iprogress,
                ),
            )))
        },
        None => Ok(Some((
            Progress::ReadyToDownload,
            State::RemovingPartially(params, compared),
        ))),
    }
}

// downloads partial info
async fn downloading_partial(
    params: UpdateParameters,
    mut compared: Compared,
    download: Download<CentralDirectoryHeader>,
    mut finished: Vec<(BytesMut, CentralDirectoryHeader, LocalFileHeader)>,
    mut afterburner: Afterburner<CentralDirectoryHeader>,
    mut progress: InternalProgressData,
) -> Result<Option<(Progress, State)>, UpdateError> {
    // we can max finish 1 download each fn call, so its okay to only add 1 download.
    const DOWNLOADS_IN_QUEUE_SPEEDUP: u32 = 5;
    if afterburner.len() < DOWNLOADS_IN_QUEUE_SPEEDUP {
        if let Some(next_partially) = remote_zip::next_partial(&params, &mut compared) {
            afterburner.start(next_partially).await
        }
    }

    let download = download
        .progress()
        .await
        .map_err(|e| UpdateError::Custom(format!("{:?}", e)))?;

    match download {
        Download::Finished(s, r) => {
            progress.progress.downloaded_bytes = compared.needs_redownload_bytes
                - compared
                    .needs_redownload
                    .iter()
                    .map(|remote| remote.fixed.compressed_size as u64)
                    .sum::<u64>();
            let pr = progress.progress.clone();

            let ndownload = afterburner.next();
            if ndownload.is_none() && afterburner.len() > 0 {
                // we are not completly finished yet, spin another dummy round for
                // afterburner to do its job, the download will stay Finished
                tokio::time::sleep(Duration::from_millis(5)).await;
                return Ok(Some((
                    Progress::InProgress(pr),
                    State::DownloadingPartially(
                        params,
                        compared,
                        Download::Finished(s, r),
                        finished,
                        afterburner,
                        progress,
                    ),
                )));
            }

            let mut bytes = match s {
                Storage::Memory(b) => b,
                _ => unreachable!(),
            };

            let local_header = match LocalFileHeader::from_buf(&mut bytes) {
                Ok(lh) => lh,
                Err(_) => return fallback(&params, false),
            };
            if !local_header.is_valid_signature() {
                return fallback(&params, false);
            }
            finished.push((bytes, r, local_header));

            if let Some(ndownload) = ndownload {
                let download =
                    ndownload.map_err(|e| UpdateError::Custom(format!("{:?}", e)))?;

                Ok(Some((
                    Progress::InProgress(pr),
                    State::DownloadingPartially(
                        params,
                        compared,
                        download,
                        finished,
                        afterburner,
                        progress,
                    ),
                )))
            } else {
                let total = finished
                    .iter()
                    .map(|e| e.1.fixed.compressed_size as u64)
                    .sum();
                let progress = InternalProgressData::new(ProgressData::new(
                    total,
                    DownloadContent::CentralDirectory,
                ));
                tracing::info!("unzipping files");
                Ok(Some((
                    Progress::InProgress(pr),
                    State::UnzipPartial(params, compared, finished, progress),
                )))
            }
        },
        Download::Progress(_r, _s, p, _p) => {
            progress.progress.content = p.progress.content.clone();
            progress.progress.bytes_per_sec = p.progress.bytes_per_sec;
            Ok(Some((
                Progress::InProgress(progress.progress.clone()),
                State::DownloadingPartially(
                    params,
                    compared,
                    Download::Progress(_r, _s, p, _p),
                    finished,
                    afterburner,
                    progress,
                ),
            )))
        },
        Download::Start(_, _, _, _) => unreachable!(),
    }
}

// downloads partial info
async fn unzip_partial(
    params: UpdateParameters,
    compared: Compared,
    mut finished: Vec<(BytesMut, CentralDirectoryHeader, LocalFileHeader)>,
    mut progress: InternalProgressData,
) -> Result<Option<(Progress, State)>, UpdateError> {
    match finished.pop() {
        Some((rbytes, remote, _)) => {
            let remote_file_size = remote.fixed.compressed_size as usize;
            if remote_file_size > rbytes.remaining() {
                tracing::warn!(
                    "Actually xMAC guessed wrong with the 1400 extra bytes, ping him \
                     please"
                );
                return fallback(&params, false);
            }

            let filename = String::from_utf8_lossy(&remote.file_name);

            let path = params.profile.directory().join(filename.to_string());
            if !path.starts_with(params.profile.directory()) {
                panic!(
                    "{}",
                    "Zip Escape Attack, it seems your zip is compromized and tries to \
                     write outside rood, call the veloren team, path tried to write to: \
                     {path:?}",
                );
            }

            let parent = path.parent().unwrap();
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|e| UpdateError::Custom(format!("{:?}", e)))?;

            let file = tokio::spawn(tokio::fs::File::create(path));

            let mut file_data = match CompressionMethod::try_from(remote.fixed.compression_method) {
                Ok(CompressionMethod::Deflated) => {
                    let compressed = rbytes.take(remote_file_size);
                    let mut deflate_reader = DeflateDecoder::new(compressed.reader());
                    let mut decompressed = Vec::with_capacity(remote_file_size);
                    deflate_reader.read_to_end(&mut decompressed).unwrap();
                    bytes::Bytes::copy_from_slice(&decompressed)
                },
                Ok(CompressionMethod::Stored) => rbytes
                    .take(remote_file_size)
                    .copy_to_bytes(remote_file_size),
                _ => return fallback(&params, false), /* should not happen at this                                         * point */
            };

            let mut file = file
                .await
                .unwrap()
                .map_err(|e| UpdateError::Custom(format!("{:?}", e)))?;
            // TODO: evaluate splitting this up
            file.write_all_buf(&mut file_data)
                .await
                .map_err(|e| UpdateError::Custom(format!("{:?}", e)))?;

            progress.add_chunk(remote_file_size as u64);

            Ok(Some((
                Progress::InProgress(progress.progress.clone()),
                State::UnzipPartial(params, compared, finished, progress),
            )))
        },
        None => {
            tracing::info!("deleting files that should be removed");
            Ok(Some((
                Progress::InProgress(progress.progress.clone()),
                State::RemovingPartially(params, compared),
            )))
        },
    }
}

// remove old files
async fn removing_partial(
    params: UpdateParameters,
    mut compared: Compared,
) -> Result<Option<(Progress, State)>, UpdateError> {
    match compared.needs_deletion.pop() {
        Some(f) => {
            tracing::debug!("deleting {:?}", &f.path);
            tokio::fs::remove_file(&f.path)
                .await
                .map_err(|e| UpdateError::Custom(format!("{:?}", e)))?;
            let progress = ProgressData {
                bytes_per_sec: 0,
                content: DownloadContent::SingleFile(f.local_path.clone()),
                total_bytes: compared.needs_deletion_total,
                downloaded_bytes: compared.needs_deletion_total
                    - compared.needs_deletion.len() as u64,
            };
            Ok(Some((
                Progress::InProgress(progress),
                State::RemovingPartially(params, compared),
            )))
        },
        None => Ok(Some((Progress::Sucessful, State::Finished))),
    }
}
