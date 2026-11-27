// SPDX-License-Identifier: GPL-3.0-only
//! Execution of commands, behind a trait so that parsers can be tested
//! with recorded output and the interface can be run without ZFS.

use crate::command::Command;
use crate::{Error, Result};

/// Runs a [`Command`] and returns its standard output.
pub trait Runner {
    fn run(&self, command: &Command) -> Result<String>;
}

/// Runs commands on the local system, with a minimal fixed environment.
#[derive(Debug, Default, Clone, Copy)]
pub struct SystemRunner;

impl Runner for SystemRunner {
    fn run(&self, command: &Command) -> Result<String> {
        let program = command.tool.program();
        let path = command.tool.locate().ok_or(Error::ToolNotFound(program))?;
        let output = std::process::Command::new(&path)
            .args(&command.args)
            .env_clear()
            .env("PATH", "/usr/sbin:/usr/bin:/sbin:/bin")
            .env("LC_ALL", "C")
            .stdin(std::process::Stdio::null())
            .output()
            .map_err(|source| Error::Spawn {
                program: program.to_owned(),
                source,
            })?;
        if !output.status.success() {
            return Err(Error::CommandFailed {
                program: program.to_owned(),
                code: output.status.code(),
                stderr: String::from_utf8_lossy(&output.stderr).trim().to_owned(),
            });
        }
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    }
}

/// A runner that replays canned output, for tests.
#[cfg(test)]
pub struct FakeRunner(pub std::collections::HashMap<Vec<String>, String>);

#[cfg(test)]
impl Runner for FakeRunner {
    fn run(&self, command: &Command) -> Result<String> {
        let mut key = vec![command.tool.program().to_owned()];
        key.extend(command.args.iter().cloned());
        self.0.get(&key).cloned().ok_or(Error::CommandFailed {
            program: key.join(" "),
            code: Some(1),
            stderr: "no canned output".into(),
        })
    }
}
