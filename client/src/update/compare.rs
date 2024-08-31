use std::collections::HashMap;

use zip_core::raw::CentralDirectoryHeader;

use super::local_directory::FileInformation;

/// Paths which should *not* be deleted by airshipper if they're not included in the
/// update
const KEEP_PATHS: &[&str] = &["userdata/", "screenshots/", "maps/", "veloren.zip"];

#[derive(Debug)]
pub(super) struct Compared {
    pub needs_redownload: Vec<CentralDirectoryHeader>,
    pub needs_deletion: Vec<FileInformation>,
    pub needs_redownload_bytes: u64,
    pub needs_deletion_total: u64,
    pub clean_data_total: u64,
}

fn extract<T, F: Fn(&T) -> bool>(input: &mut Vec<T>, f: F) -> Vec<T> {
    input.sort_by_key(&f); // [false, false, true]
    if let Some(id) = input.iter().position(f) {
        input.split_off(id)
    } else {
        vec![]
    }
}

pub(super) fn prepare_local_with_remote(
    remote: Vec<CentralDirectoryHeader>,
    local: Vec<FileInformation>,
) -> Compared {
    let mut compares = compare_local_with_remote(remote, local);

    let clean = extract(&mut compares, |e| matches!(e, CompareEntry::EqualCrc(_, _)));
    let needs_deletion: Vec<_> = extract(&mut compares, |e| {
        matches!(e, CompareEntry::ExistsInLocal(_))
    })
    .into_iter()
    .filter_map(|e| match e {
        CompareEntry::ExistsInLocal(local) => Some(local),
        _ => None,
    })
    .collect();
    let mut needs_redownload: Vec<_> = compares
        .into_iter()
        .filter_map(|e| match e {
            CompareEntry::ExistsInRemote(remote) => Some(remote),
            CompareEntry::DifferentCrc(remote, _) => Some(remote),
            _ => None,
        })
        .collect();

    let clean_data_total = clean
        .into_iter()
        .map(|e| match e {
            CompareEntry::EqualCrc(remote, _) => remote.fixed.compressed_size as u64,
            _ => 0,
        })
        .sum();
    let needs_redownload_bytes = needs_redownload
        .iter()
        .map(|remote| remote.fixed.compressed_size as u64)
        .sum();

    // keep everything in userdata

    let needs_deletion: Vec<_> = needs_deletion
        .into_iter()
        .filter(|fi| {
            !KEEP_PATHS
                .into_iter()
                .any(|keep| fi.local_unix_path.starts_with(keep))
        })
        .collect();

    let needs_deletion_total = needs_deletion.len() as u64;

    //reorder based by range, so that we read from low to high, in the hope that its
    // better for the remote spinning disk. but reversed, because we .pop from this Vec
    needs_redownload.sort_by_key(|e| e.fixed.relative_offset_of_local_header);
    needs_redownload.reverse();

    Compared {
        needs_redownload,
        needs_deletion,
        needs_redownload_bytes,
        needs_deletion_total,
        clean_data_total,
    }
}

#[allow(dead_code)]
enum CompareEntry {
    ExistsInRemote(CentralDirectoryHeader),
    ExistsInLocal(FileInformation),
    EqualCrc(CentralDirectoryHeader, FileInformation),
    DifferentCrc(CentralDirectoryHeader, FileInformation),
}

fn compare_local_with_remote(
    remote: Vec<CentralDirectoryHeader>,
    local: Vec<FileInformation>,
) -> Vec<CompareEntry> {
    let mut compare_map: HashMap<
        String,
        (Option<FileInformation>, Option<CentralDirectoryHeader>),
    > = HashMap::new();

    for l in local {
        let _ = compare_map
            .entry(l.local_unix_path.clone())
            .or_insert((Some(l), None));
    }
    for r in remote {
        let file_name = std::str::from_utf8(&r.file_name).unwrap().to_string();
        let e = compare_map.entry(file_name).or_insert((None, None));
        e.1 = Some(r);
    }

    compare_map
        .into_values()
        .flat_map(|d| match (d.0, d.1) {
            (None, Some(remote)) => {
                // check if dir
                if remote.fixed.uncompressed_size == 0 {
                    None
                } else {
                    Some(CompareEntry::ExistsInRemote(remote))
                }
            },
            (Some(local), None) => Some(CompareEntry::ExistsInLocal(local)),
            (None, None) => unreachable!(),
            (Some(local), Some(remote)) => {
                if local.crc32 == remote.fixed.crc_32 {
                    Some(CompareEntry::EqualCrc(remote, local))
                } else {
                    Some(CompareEntry::DifferentCrc(remote, local))
                }
            },
        })
        .collect()
}
