// SPDX-License-Identifier: GPL-3.0-only
//! Detection of the installed OpenZFS version.

use crate::command;
use crate::runner::Runner;
use crate::Result;

/// Versions reported by `zfs version`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ZfsVersion {
    /// Version of the user-space tools, e.g. `2.2.2-0ubuntu9`.
    pub userland: String,
    /// Version of the loaded kernel module, if the module is loaded.
    pub kernel: Option<String>,
}

/// Parses the output of `zfs version`:
///
/// ```text
/// zfs-2.2.2-0ubuntu9
/// zfs-kmod-2.2.2-0ubuntu9
/// ```
pub fn parse_version(output: &str) -> Option<ZfsVersion> {
    let mut userland = None;
    let mut kernel = None;
    for line in output.lines().map(str::trim) {
        if let Some(v) = line.strip_prefix("zfs-kmod-") {
            kernel = Some(v.to_owned());
        } else if let Some(v) = line.strip_prefix("zfs-") {
            userland = Some(v.to_owned());
        }
    }
    Some(ZfsVersion {
        userland: userland?,
        kernel,
    })
}

/// Runs `zfs version` and parses its output.
pub fn detect(runner: &impl Runner) -> Result<Option<ZfsVersion>> {
    let output = runner.run(&command::version())?;
    Ok(parse_version(&output))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_both_lines() {
        let v = parse_version("zfs-2.2.2-0ubuntu9\nzfs-kmod-2.2.2-0ubuntu9\n").unwrap();
        assert_eq!(v.userland, "2.2.2-0ubuntu9");
        assert_eq!(v.kernel.as_deref(), Some("2.2.2-0ubuntu9"));
    }

    #[test]
    fn module_not_loaded() {
        let v = parse_version("zfs-2.1.11-1\n").unwrap();
        assert_eq!(v.kernel, None);
        assert_eq!(parse_version("garbage"), None);
    }
}
