// SPDX-License-Identifier: GPL-3.0-or-later
//! Runs the privileged helper through `pkexec`.

use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

use gettextrs::gettext;
use zfs_core::helper::{Envelope, ErrorCode, Request, Response};

use crate::config;
use crate::i18n::gettext_f;

/// Why a privileged operation did not run.
#[derive(Debug)]
pub enum HelperError {
    /// The user closed the authentication dialog.
    Cancelled,
    /// Authentication failed, or polkit refused the action.
    NotAuthorized,
    /// `pkexec` or the helper is missing.
    NotInstalled(String),
    /// The helper ran and reported an error.
    Failed { code: ErrorCode, detail: String },
    /// Anything else (I/O error, unreadable answer).
    Other(String),
}

impl HelperError {
    /// A translated message for the user.
    pub fn message(&self) -> String {
        match self {
            HelperError::Cancelled => gettext("Authentication was cancelled."),
            HelperError::NotAuthorized => gettext("Administrator authentication failed."),
            HelperError::NotInstalled(path) => {
                gettext_f("The administration helper is not installed ({}).", &[path])
            }
            HelperError::Failed { code, detail } => {
                let reason = match code {
                    ErrorCode::InvalidRequest => gettext("The request was rejected."),
                    ErrorCode::InvalidName => gettext("The name is not valid."),
                    ErrorCode::NotAuthorized => gettext("Administrator rights are required."),
                    ErrorCode::ToolNotFound => gettext("OpenZFS is not installed."),
                    ErrorCode::CommandFailed => gettext("The ZFS command failed."),
                };
                if detail.is_empty() {
                    reason
                } else {
                    format!("{reason}\n{detail}")
                }
            }
            HelperError::Other(detail) => gettext_f("Unexpected error: {}", &[detail]),
        }
    }
}

fn locate_pkexec() -> Option<&'static Path> {
    ["/usr/bin/pkexec", "/bin/pkexec"]
        .into_iter()
        .map(Path::new)
        .find(|p| p.is_file())
}

/// Sends one request to the helper and waits for its answer.
///
/// This blocks while the authentication dialog is open: call it from
/// `gio::spawn_blocking`, never from the GTK main thread.
pub fn call(request: Request) -> Result<String, HelperError> {
    let pkexec = locate_pkexec().ok_or_else(|| HelperError::NotInstalled("pkexec".into()))?;
    let helper = config::helper_path();
    if !helper.is_file() {
        return Err(HelperError::NotInstalled(helper.display().to_string()));
    }

    let mut child = Command::new(pkexec)
        .arg(&helper)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| HelperError::Other(e.to_string()))?;

    let payload = serde_json::to_vec(&Envelope::new(request))
        .map_err(|e| HelperError::Other(e.to_string()))?;
    if let Some(mut stdin) = child.stdin.take() {
        // A write error means the helper exited early; its exit code says why.
        let _ = stdin.write_all(&payload);
    }

    let output = child
        .wait_with_output()
        .map_err(|e| HelperError::Other(e.to_string()))?;

    if let Ok(response) = serde_json::from_slice::<Response>(&output.stdout) {
        return match response {
            Response::Ok { output } => Ok(output),
            Response::Error { code, detail } => Err(HelperError::Failed { code, detail }),
        };
    }
    // No answer from the helper: pkexec itself stopped.
    match output.status.code() {
        Some(126) => Err(HelperError::Cancelled),
        Some(127) => Err(HelperError::NotAuthorized),
        _ => Err(HelperError::Other(
            String::from_utf8_lossy(&output.stderr).trim().to_owned(),
        )),
    }
}
