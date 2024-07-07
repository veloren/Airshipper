use std::collections::HashMap;

use zip_core::raw::CentralDirectoryHeader;

use crate::local_directory::FileInformation;

#[derive(Clone, Debug)]
pub enum CompareEntry {
    ExistsInRemote(CentralDirectoryHeader),
    ExistsInLocal(FileInformation),
    EqualCrc(CentralDirectoryHeader, FileInformation),
    DifferentCrc(CentralDirectoryHeader, FileInformation),
}

pub fn compare_local_with_remote(
    remote: Vec<CentralDirectoryHeader>,
    local: Vec<FileInformation>,
) -> Vec<CompareEntry> {
    let mut compare_map: HashMap<
        String,
        (Option<FileInformation>, Option<CentralDirectoryHeader>),
    > = HashMap::new();

    for l in local {
        let _ = compare_map
            .entry(l.local_path.clone())
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
