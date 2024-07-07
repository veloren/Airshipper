use zip_core::{raw::CentralDirectoryHeader, structs::CompressionMethod};

use crate::compare::CompareEntry;

pub(crate) fn compression_supported(
    cd: &CentralDirectoryHeader,
) -> Option<CompressionMethod> {
    match CompressionMethod::try_from(cd.fixed.compression_method) {
        Ok(CompressionMethod::Deflated) => Some(CompressionMethod::Deflated),
        Ok(CompressionMethod::Stored) => Some(CompressionMethod::Stored),
        _ => None,
    }
}

pub fn compression_method_supported(entries: &[CompareEntry]) -> bool {
    !entries.iter().any(|e| {
        let cd = match e {
            CompareEntry::ExistsInRemote(remote) => remote,
            CompareEntry::DifferentCrc(remote, _) => remote,
            _ => return false,
        };
        compression_supported(cd).is_none()
    })
}
