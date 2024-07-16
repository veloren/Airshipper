use crate::{error::ClientError, profiles::Profile, GITHUB_CLIENT, WEB_CLIENT};
use afterburner::Afterburner;
use compare::{prepare_local_with_remote, Compared};
use download::{Download, DownloadContent, InternalProgressData, ProgressData, Storage};
use futures_util::stream::Stream;
use iced::futures;
use local_directory::{FileInformation, LocalDirectory};
use remote_zip::{gen_classsic, RemoteZipError};
use zip_core::{raw::CentralDirectoryHeader, structs::CompressionMethod};

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
    DownloadCentralDirectory(UpdateParameters, bool, Download),
    CrcLocalDirectory(
        UpdateParameters,
        LocalDirectory,
        Vec<CentralDirectoryHeader>,
    ),
    DownloadingClassic(UpdateParameters, Download),
    FilterMissingFiles(
        UpdateParameters,
        Vec<CentralDirectoryHeader>,
        Vec<FileInformation>,
    ),
    DownloadingPartially(
        UpdateParameters,
        Compared,
        Download,
        Afterburner,
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
            State::DownloadingClassic(p, d) => downloading_classing(p, d).await,
            State::FilterMissingFiles(p, cds, fi) => filter_missings(p, cds, fi).await,
            State::DownloadingPartially(p, cp, d, a, pr) => {
                downloading_partial(p, cp, d, a, pr).await
            },
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
    download: Download,
) -> Result<Option<(Progress, State)>, UpdateError> {
    let download = download
        .progress()
        .await
        .map_err(|e| UpdateError::Custom(format!("{:?}", e)))?;
    match download {
        Download::Finished(storage) => {
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
        Download::Progress(_, _, _) => Ok(Some((
            Progress::Evaluating,
            State::DownloadCentralDirectory(params, remote_matches_profile, download),
        ))),
        Download::Start(_, _, _) => unreachable!(),
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
async fn downloading_classing(
    params: UpdateParameters,
    download: Download,
) -> Result<Option<(Progress, State)>, UpdateError> {
    let download = download
        .progress()
        .await
        .map_err(|e| UpdateError::Custom(format!("{:?}", e)))?;
    match &download {
        Download::Finished(_) => Ok(Some((Progress::Sucessful, State::Finished))),
        Download::Progress(_, _, progress) => Ok(Some((
            Progress::InProgress(progress.progress.clone()),
            State::DownloadingClassic(params, download),
        ))),
        Download::Start(_, _, _) => unreachable!(),
    }
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
    tracing::debug!("need to delet {} files", compared.needs_deletion_total);
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

    const MAX_FILES_FOR_PARTIAL_DOWNLOAD: usize = 350;
    if compared.needs_redownload.len() > MAX_FILES_FOR_PARTIAL_DOWNLOAD {
        tracing::info!(
            "to many files changed to make partial download efficient, falling back to \
             classic mode"
        );
        return fallback(&params, false);
    }

    // create backup dir

    let progress = ProgressData {
        bytes_per_sec: 0,
        downloaded_bytes: 0,
        total_bytes: compared.needs_redownload_bytes,
        content: DownloadContent::CentralDirectory,
    };
    let iprogress = InternalProgressData::new(progress);

    match remote_zip::next_partial(&params, &mut compared) {
        Some(first_partially) => Ok(Some((
            Progress::ReadyToDownload,
            State::DownloadingPartially(
                params,
                compared,
                first_partially,
                Afterburner::default(),
                iprogress,
            ),
        ))),
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
    download: Download,
    mut afterburner: Afterburner,
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

    match &download {
        Download::Finished(_) => {
            progress.progress.downloaded_bytes = compared.needs_redownload_bytes
                - compared
                    .needs_redownload
                    .iter()
                    .map(|remote| remote.fixed.compressed_size as u64)
                    .sum::<u64>();
            let pr = progress.progress.clone();

            if let Some(download) = afterburner.next() {
                let download =
                    download.map_err(|e| UpdateError::Custom(format!("{:?}", e)))?;
                Ok(Some((
                    Progress::InProgress(pr),
                    State::DownloadingPartially(
                        params,
                        compared,
                        download,
                        afterburner,
                        progress,
                    ),
                )))
            } else if afterburner.len() == 0 {
                Ok(Some((
                    Progress::InProgress(pr),
                    State::RemovingPartially(params, compared),
                )))
            } else {
                // If we are finished, no download from afterburner is ready but some are
                // in queue, we just do another dummy round without blocking waiting for
                // the afterburner
                Ok(Some((
                    Progress::InProgress(pr),
                    State::DownloadingPartially(
                        params,
                        compared,
                        download,
                        afterburner,
                        progress,
                    ),
                )))
            }
        },
        Download::Progress(_, _, p) => {
            progress.progress.content = p.progress.content.clone();
            progress.progress.bytes_per_sec = p.progress.bytes_per_sec;
            Ok(Some((
                Progress::InProgress(progress.progress.clone()),
                State::DownloadingPartially(
                    params,
                    compared,
                    download,
                    afterburner,
                    progress,
                ),
            )))
        },
        Download::Start(_, _, _) => unreachable!(),
    }
}

// remove old files
async fn removing_partial(
    params: UpdateParameters,
    mut compared: Compared,
) -> Result<Option<(Progress, State)>, UpdateError> {
    match compared.needs_deletion.pop() {
        Some(f) => {
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

/*
 - Download single zippy while compressing latest (assuming download takes longer than decompression, speedup)
 - after confirm, do a backup step, write changes to the clone repo, and mv directories later on.

*/
