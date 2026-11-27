// SPDX-License-Identifier: GPL-3.0-or-later
//! Storage pools, as listed by `zpool list -H -p`.

use crate::command::{self, POOL_COLUMNS};
use crate::runner::Runner;
use crate::{Error, Result};

/// Health of a pool or a device.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Health {
    Online,
    Degraded,
    Faulted,
    Offline,
    Unavail,
    Removed,
    Suspended,
    Other(String),
}

impl Health {
    pub fn parse(value: &str) -> Self {
        match value {
            "ONLINE" => Health::Online,
            "DEGRADED" => Health::Degraded,
            "FAULTED" => Health::Faulted,
            "OFFLINE" => Health::Offline,
            "UNAVAIL" => Health::Unavail,
            "REMOVED" => Health::Removed,
            "SUSPENDED" => Health::Suspended,
            other => Health::Other(other.to_owned()),
        }
    }

    /// Whether the pool needs attention.
    pub fn is_problem(&self) -> bool {
        !matches!(self, Health::Online)
    }
}

/// One line of `zpool list`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pool {
    pub name: String,
    pub size: u64,
    pub allocated: u64,
    pub free: u64,
    /// Free-space fragmentation in percent, when the pool reports it.
    pub fragmentation: Option<u8>,
    /// Used capacity in percent.
    pub capacity: u8,
    pub health: Health,
}

/// Parses a numeric field, where `-` means "not available".
pub(crate) fn parse_optional<T: std::str::FromStr>(value: &str) -> Option<T> {
    match value {
        "-" | "" => None,
        v => v.trim_end_matches('%').parse().ok(),
    }
}

pub(crate) fn parse_required<T: std::str::FromStr>(
    program: &'static str,
    line: usize,
    column: &str,
    value: &str,
) -> Result<T> {
    parse_optional(value).ok_or_else(|| Error::Parse {
        program,
        line,
        reason: format!("invalid value `{value}` for `{column}`"),
    })
}

/// Parses the output of [`command::list_pools`].
pub fn parse_pool_list(output: &str) -> Result<Vec<Pool>> {
    let expected = POOL_COLUMNS.split(',').count();
    output
        .lines()
        .filter(|l| !l.trim().is_empty())
        .enumerate()
        .map(|(i, line)| {
            let n = i + 1;
            let f: Vec<&str> = line.split('\t').collect();
            if f.len() != expected {
                return Err(Error::Parse {
                    program: "zpool",
                    line: n,
                    reason: format!("expected {expected} columns, found {}", f.len()),
                });
            }
            Ok(Pool {
                name: f[0].to_owned(),
                size: parse_required("zpool", n, "size", f[1])?,
                allocated: parse_required("zpool", n, "allocated", f[2])?,
                free: parse_required("zpool", n, "free", f[3])?,
                fragmentation: parse_optional(f[4]),
                capacity: parse_required("zpool", n, "capacity", f[5])?,
                health: Health::parse(f[6]),
            })
        })
        .collect()
}

/// Lists the imported pools.
pub fn list(runner: &impl Runner) -> Result<Vec<Pool>> {
    parse_pool_list(&runner.run(&command::list_pools())?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runner::FakeRunner;

    const SAMPLE: &str = "rpool\t1992864825344\t412316860416\t1580547964928\t7\t20\tONLINE\n\
                          backup\t3985729650688\t3587156230144\t398573420544\t-\t90\tDEGRADED\n";

    #[test]
    fn parses_pools() {
        let pools = parse_pool_list(SAMPLE).unwrap();
        assert_eq!(pools.len(), 2);
        assert_eq!(pools[0].name, "rpool");
        assert_eq!(pools[0].fragmentation, Some(7));
        assert_eq!(pools[0].capacity, 20);
        assert_eq!(pools[1].fragmentation, None);
        assert_eq!(pools[1].health, Health::Degraded);
        assert!(pools[1].health.is_problem());
    }

    #[test]
    fn empty_output_means_no_pools() {
        assert!(parse_pool_list("").unwrap().is_empty());
    }

    #[test]
    fn rejects_malformed_lines() {
        assert!(parse_pool_list("rpool\t12\n").is_err());
        assert!(parse_pool_list("rpool\tx\t1\t1\t1\t1\tONLINE\n").is_err());
    }

    #[test]
    fn lists_through_runner() {
        let mut key = vec!["zpool".to_owned()];
        key.extend(command::list_pools().args);
        let runner = FakeRunner([(key, SAMPLE.to_owned())].into());
        assert_eq!(list(&runner).unwrap().len(), 2);
    }
}
