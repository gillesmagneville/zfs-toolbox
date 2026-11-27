// SPDX-License-Identifier: GPL-3.0-only
//! Builders for the `zfs` and `zpool` command lines used by the application.
//!
//! Commands are always built as an argument vector and executed without a
//! shell, so names are never subject to word splitting or expansion.

use std::path::{Path, PathBuf};

/// One of the two OpenZFS administration tools.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tool {
    Zfs,
    Zpool,
}

impl Tool {
    pub fn program(self) -> &'static str {
        match self {
            Tool::Zfs => "zfs",
            Tool::Zpool => "zpool",
        }
    }

    /// Finds the tool in the standard system directories.
    ///
    /// The search path is fixed rather than taken from `$PATH`, because the
    /// privileged helper must never run a program chosen by the caller.
    pub fn locate(self) -> Option<PathBuf> {
        const DIRS: &[&str] = &["/usr/sbin", "/sbin", "/usr/bin", "/bin", "/usr/local/sbin"];
        DIRS.iter()
            .map(|dir| Path::new(dir).join(self.program()))
            .find(|path| path.is_file())
    }
}

/// A command ready to be run: a tool and its arguments.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Command {
    pub tool: Tool,
    pub args: Vec<String>,
}

impl Command {
    fn new(tool: Tool, args: &[&str]) -> Self {
        Self {
            tool,
            args: args.iter().map(|a| (*a).to_owned()).collect(),
        }
    }
}

/// Columns requested from `zpool list`, in the order parsed by [`crate::pool`].
pub const POOL_COLUMNS: &str = "name,size,allocated,free,fragmentation,capacity,health";

/// Columns requested from `zfs list`, in the order parsed by [`crate::dataset`].
pub const DATASET_COLUMNS: &str =
    "name,type,used,available,referenced,mountpoint,mounted,encryption,keystatus";

/// `zfs version`
pub fn version() -> Command {
    Command::new(Tool::Zfs, &["version"])
}

/// `zpool list` in script mode with exact byte values.
pub fn list_pools() -> Command {
    Command::new(Tool::Zpool, &["list", "-H", "-p", "-o", POOL_COLUMNS])
}

/// `zfs list` of every filesystem and volume, in script mode.
pub fn list_datasets() -> Command {
    Command::new(
        Tool::Zfs,
        &[
            "list",
            "-H",
            "-p",
            "-t",
            "filesystem,volume",
            "-o",
            DATASET_COLUMNS,
        ],
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_script_mode_commands() {
        let cmd = list_pools();
        assert_eq!(cmd.tool, Tool::Zpool);
        assert_eq!(cmd.args[..3], ["list", "-H", "-p"]);
        assert_eq!(list_datasets().tool.program(), "zfs");
    }
}
