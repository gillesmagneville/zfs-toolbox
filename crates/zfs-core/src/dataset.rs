// SPDX-License-Identifier: GPL-3.0-or-later
//! Filesystems and volumes, as listed by `zfs list -H -p`.

use crate::command::{self, DATASET_COLUMNS};
use crate::pool::parse_required;
use crate::runner::Runner;
use crate::{Error, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DatasetKind {
    Filesystem,
    Volume,
}

/// State of the encryption key of an encrypted dataset.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyStatus {
    /// The dataset is not encrypted.
    None,
    Available,
    Unavailable,
}

/// One line of `zfs list`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dataset {
    pub name: String,
    pub kind: DatasetKind,
    pub used: u64,
    pub available: u64,
    pub referenced: u64,
    /// Mount point, or `None` for volumes and `none`/`legacy`/`-` values.
    pub mountpoint: Option<String>,
    pub mounted: bool,
    /// Encryption algorithm, or `None` when not encrypted.
    pub encryption: Option<String>,
    pub key_status: KeyStatus,
}

impl Dataset {
    /// Name of the pool that holds this dataset.
    pub fn pool(&self) -> &str {
        self.name.split('/').next().unwrap_or(&self.name)
    }

    /// Nesting depth: 0 for the pool's root dataset.
    pub fn depth(&self) -> usize {
        self.name.matches('/').count()
    }
}

/// Parses the output of [`command::list_datasets`].
pub fn parse_dataset_list(output: &str) -> Result<Vec<Dataset>> {
    let expected = DATASET_COLUMNS.split(',').count();
    output
        .lines()
        .filter(|l| !l.trim().is_empty())
        .enumerate()
        .map(|(i, line)| {
            let n = i + 1;
            let f: Vec<&str> = line.split('\t').collect();
            if f.len() != expected {
                return Err(Error::Parse {
                    program: "zfs",
                    line: n,
                    reason: format!("expected {expected} columns, found {}", f.len()),
                });
            }
            let kind = match f[1] {
                "filesystem" => DatasetKind::Filesystem,
                "volume" => DatasetKind::Volume,
                other => {
                    return Err(Error::Parse {
                        program: "zfs",
                        line: n,
                        reason: format!("unknown dataset type `{other}`"),
                    })
                }
            };
            let mountpoint = match f[5] {
                "-" | "none" | "legacy" => None,
                path => Some(path.to_owned()),
            };
            let encryption = match f[7] {
                "-" | "off" => None,
                algo => Some(algo.to_owned()),
            };
            let key_status = match f[8] {
                "available" => KeyStatus::Available,
                "unavailable" => KeyStatus::Unavailable,
                _ => KeyStatus::None,
            };
            Ok(Dataset {
                name: f[0].to_owned(),
                kind,
                used: parse_required("zfs", n, "used", f[2])?,
                available: parse_required("zfs", n, "available", f[3])?,
                referenced: parse_required("zfs", n, "referenced", f[4])?,
                mountpoint,
                mounted: f[6] == "yes",
                encryption,
                key_status,
            })
        })
        .collect()
}

/// Lists every filesystem and volume of the imported pools.
pub fn list(runner: &impl Runner) -> Result<Vec<Dataset>> {
    parse_dataset_list(&runner.run(&command::list_datasets())?)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "\
rpool\tfilesystem\t412316860416\t1500000000000\t98304\t/\tno\toff\t-
rpool/home\tfilesystem\t20000000\t1500000000000\t20000000\t/home\tyes\toff\t-
rpool/secret\tfilesystem\t4096\t1500000000000\t4096\t/secret\tno\taes-256-gcm\tunavailable
rpool/vm\tvolume\t10737418240\t1500000000000\t5000000\t-\t-\toff\t-
";

    #[test]
    fn parses_datasets() {
        let ds = parse_dataset_list(SAMPLE).unwrap();
        assert_eq!(ds.len(), 4);
        assert_eq!(ds[1].mountpoint.as_deref(), Some("/home"));
        assert!(ds[1].mounted);
        assert_eq!(ds[1].depth(), 1);
        assert_eq!(ds[2].encryption.as_deref(), Some("aes-256-gcm"));
        assert_eq!(ds[2].key_status, KeyStatus::Unavailable);
        assert_eq!(ds[3].kind, DatasetKind::Volume);
        assert_eq!(ds[3].mountpoint, None);
        assert_eq!(ds[3].pool(), "rpool");
    }

    #[test]
    fn rejects_unknown_type() {
        let line = "rpool@s\tsnapshot\t1\t1\t1\t-\t-\toff\t-\n";
        assert!(parse_dataset_list(line).is_err());
    }
}
