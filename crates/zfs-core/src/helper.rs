// SPDX-License-Identifier: GPL-3.0-only
//! Protocol between the application and its privileged helper.
//!
//! The application runs as the logged-in user. Operations that change the
//! system (creating a pool, loading a key, ...) are sent to
//! `zfs-toolbox-helper`, started through `pkexec` so that polkit asks for
//! administrator authentication.
//!
//! The application writes one JSON [`Envelope`] on the helper's standard
//! input and reads one JSON [`Response`] from its standard output. Only the
//! operations listed in [`Request`] exist: the helper never runs an
//! arbitrary command line.

use serde::{Deserialize, Serialize};

/// Bumped whenever [`Request`] or [`Response`] change incompatibly.
pub const PROTOCOL_VERSION: u32 = 1;

/// Requests larger than this are rejected without being parsed.
pub const MAX_REQUEST_BYTES: usize = 64 * 1024;

/// Polkit action that authorizes running the helper.
pub const POLKIT_ACTION: &str = "io.github.gillesmagneville.ZfsToolbox.manage";

/// What the helper reads on its standard input.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Envelope {
    pub version: u32,
    pub request: Request,
}

impl Envelope {
    pub fn new(request: Request) -> Self {
        Self {
            version: PROTOCOL_VERSION,
            request,
        }
    }
}

/// An operation the helper may perform as root.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "kebab-case", deny_unknown_fields)]
pub enum Request {
    /// Checks that the helper runs and has administrator rights.
    Ping {},
}

impl Request {
    /// Validates every name carried by the request.
    pub fn validate(&self) -> crate::Result<()> {
        match self {
            Request::Ping {} => Ok(()),
        }
    }
}

/// Machine-readable error kinds, translated by the application.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ErrorCode {
    /// The request could not be read or has an unsupported version.
    InvalidRequest,
    /// A pool, dataset or snapshot name was rejected.
    InvalidName,
    /// The helper is not running with administrator rights.
    NotAuthorized,
    /// `zfs` or `zpool` is not installed.
    ToolNotFound,
    /// `zfs` or `zpool` reported an error.
    CommandFailed,
}

/// What the helper writes on its standard output.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "kebab-case")]
pub enum Response {
    Ok {
        #[serde(default)]
        output: String,
    },
    Error {
        code: ErrorCode,
        detail: String,
    },
}

impl Response {
    pub fn error(code: ErrorCode, detail: impl Into<String>) -> Self {
        Response::Error {
            code,
            detail: detail.into(),
        }
    }
}

impl From<&crate::Error> for ErrorCode {
    fn from(error: &crate::Error) -> Self {
        use crate::Error as E;
        match error {
            E::InvalidName { .. } => ErrorCode::InvalidName,
            E::ToolNotFound(_) => ErrorCode::ToolNotFound,
            E::Spawn { .. } | E::CommandFailed { .. } | E::Parse { .. } => ErrorCode::CommandFailed,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wire_format_is_stable() {
        let json = serde_json::to_string(&Envelope::new(Request::Ping {})).unwrap();
        assert_eq!(json, r#"{"version":1,"request":{"op":"ping"}}"#);

        let ok: Response = serde_json::from_str(r#"{"status":"ok","output":"uid=0"}"#).unwrap();
        assert_eq!(
            ok,
            Response::Ok {
                output: "uid=0".into()
            }
        );

        let err = serde_json::to_string(&Response::error(ErrorCode::NotAuthorized, "x")).unwrap();
        assert_eq!(
            err,
            r#"{"status":"error","code":"not-authorized","detail":"x"}"#
        );
    }

    #[test]
    fn rejects_unknown_operations() {
        assert!(
            serde_json::from_str::<Envelope>(r#"{"version":1,"request":{"op":"rm"}}"#).is_err()
        );
        assert!(
            serde_json::from_str::<Envelope>(r#"{"version":1,"request":{"op":"ping","x":1}}"#)
                .is_err()
        );
    }
}
