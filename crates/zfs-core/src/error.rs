// SPDX-License-Identifier: GPL-3.0-only
use thiserror::Error;

/// Errors raised by the core library.
#[derive(Debug, Error)]
pub enum Error {
    /// The `zfs` or `zpool` program could not be found.
    #[error("the `{0}` command was not found; is OpenZFS installed?")]
    ToolNotFound(&'static str),

    /// The program could not be started.
    #[error("failed to run `{program}`: {source}")]
    Spawn {
        program: String,
        #[source]
        source: std::io::Error,
    },

    /// The program ran but reported a failure.
    #[error("`{program}` failed (exit code {code:?}): {stderr}")]
    CommandFailed {
        program: String,
        code: Option<i32>,
        stderr: String,
    },

    /// The program's output did not have the expected shape.
    #[error("unexpected output from `{program}` at line {line}: {reason}")]
    Parse {
        program: &'static str,
        line: usize,
        reason: String,
    },

    /// A pool, dataset or snapshot name was rejected.
    #[error("invalid name `{name}`: {reason}")]
    InvalidName { name: String, reason: &'static str },
}

pub type Result<T> = std::result::Result<T, Error>;
