// SPDX-License-Identifier: GPL-3.0-only
//! `zfs-toolbox-helper`: performs the privileged operations of ZFS Toolbox.
//!
//! It is started by the application through `pkexec`, which asks polkit to
//! authenticate an administrator (action
//! `io.github.gillesmagneville.ZfsToolbox.manage`). It reads exactly one
//! request on standard input, performs it, writes one response on standard
//! output and exits.
//!
//! Security rules:
//! - only the operations of `zfs_core::helper::Request` exist;
//! - every name is validated before use;
//! - `zfs`/`zpool` are looked up in fixed system directories and run without
//!   a shell, with a cleared environment.

use std::io::{Read, Write};
use std::process::ExitCode;

use zfs_core::helper::{
    Envelope, ErrorCode, Request, Response, MAX_REQUEST_BYTES, PROTOCOL_VERSION,
};

fn read_request() -> Result<Envelope, Response> {
    let mut buffer = Vec::new();
    std::io::stdin()
        .take(MAX_REQUEST_BYTES as u64 + 1)
        .read_to_end(&mut buffer)
        .map_err(|e| Response::error(ErrorCode::InvalidRequest, e.to_string()))?;
    if buffer.len() > MAX_REQUEST_BYTES {
        return Err(Response::error(
            ErrorCode::InvalidRequest,
            "request too large",
        ));
    }
    let envelope: Envelope = serde_json::from_slice(&buffer)
        .map_err(|e| Response::error(ErrorCode::InvalidRequest, e.to_string()))?;
    if envelope.version != PROTOCOL_VERSION {
        return Err(Response::error(
            ErrorCode::InvalidRequest,
            format!(
                "protocol version {} is not supported (expected {PROTOCOL_VERSION})",
                envelope.version
            ),
        ));
    }
    Ok(envelope)
}

fn handle(request: &Request) -> Response {
    if let Err(error) = request.validate() {
        return Response::error(ErrorCode::from(&error), error.to_string());
    }
    match request {
        Request::Ping {} => Response::Ok {
            // SAFETY: geteuid has no preconditions and cannot fail.
            output: format!("uid={}", unsafe { libc::geteuid() }),
        },
    }
}

fn main() -> ExitCode {
    // SAFETY: geteuid has no preconditions and cannot fail.
    let response = if unsafe { libc::geteuid() } != 0 {
        Response::error(
            ErrorCode::NotAuthorized,
            "zfs-toolbox-helper must be started through pkexec",
        )
    } else {
        match read_request() {
            Ok(envelope) => handle(&envelope.request),
            Err(response) => response,
        }
    };

    let mut stdout = std::io::stdout().lock();
    let written = serde_json::to_writer(&mut stdout, &response)
        .map_err(std::io::Error::from)
        .and_then(|()| stdout.write_all(b"\n"))
        .and_then(|()| stdout.flush());
    match (written, &response) {
        (Err(_), _) => ExitCode::from(2),
        (Ok(()), Response::Ok { .. }) => ExitCode::SUCCESS,
        (Ok(()), Response::Error { .. }) => ExitCode::from(1),
    }
}
