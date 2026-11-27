// SPDX-License-Identifier: GPL-3.0-or-later
//! Validation of pool, dataset and snapshot names.
//!
//! These checks follow the OpenZFS naming rules and are deliberately strict:
//! the privileged helper relies on them so that a name can never be
//! interpreted as a command-line option or smuggle in another argument.

use crate::{Error, Result};

/// Maximum length of a full name, as defined by `ZFS_MAX_DATASET_NAME_LEN`
/// (256 bytes including the terminating NUL).
pub const MAX_NAME_LEN: usize = 255;

/// Pool names that are reserved by OpenZFS, or prefixes of reserved names.
const RESERVED_POOL_PREFIXES: &[&str] = &["mirror", "raidz", "draid", "spare"];
const RESERVED_POOL_NAMES: &[&str] = &["log"];

fn invalid(name: &str, reason: &'static str) -> Error {
    Error::InvalidName {
        name: name.to_owned(),
        reason,
    }
}

fn is_component_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.' | ':' | ' ')
}

/// Checks one path component (between slashes, or after `@`).
fn check_component(full: &str, component: &str) -> Result<()> {
    if component.is_empty() {
        return Err(invalid(full, "empty component"));
    }
    if component == "." || component == ".." {
        return Err(invalid(full, "`.` and `..` are not allowed"));
    }
    if !component.chars().all(is_component_char) {
        return Err(invalid(full, "contains a forbidden character"));
    }
    Ok(())
}

fn check_length(name: &str) -> Result<()> {
    if name.is_empty() {
        return Err(invalid(name, "name is empty"));
    }
    if name.len() > MAX_NAME_LEN {
        return Err(invalid(name, "name is too long"));
    }
    Ok(())
}

/// Validates a pool name such as `tank`.
pub fn validate_pool_name(name: &str) -> Result<()> {
    check_length(name)?;
    if !name.starts_with(|c: char| c.is_ascii_alphabetic()) {
        return Err(invalid(name, "a pool name must start with a letter"));
    }
    if name.contains(' ') {
        return Err(invalid(name, "a pool name cannot contain spaces"));
    }
    check_component(name, name)?;
    let lower = name.to_ascii_lowercase();
    if RESERVED_POOL_NAMES.contains(&lower.as_str())
        || RESERVED_POOL_PREFIXES.iter().any(|p| lower.starts_with(p))
    {
        return Err(invalid(name, "this pool name is reserved"));
    }
    Ok(())
}

/// Validates a filesystem or volume name such as `tank/home/gilles`.
pub fn validate_dataset_name(name: &str) -> Result<()> {
    check_length(name)?;
    if name.contains(['@', '#']) {
        return Err(invalid(name, "a dataset name cannot contain `@` or `#`"));
    }
    let mut parts = name.split('/');
    let pool = parts.next().unwrap_or_default();
    validate_pool_name(pool).map_err(|_| invalid(name, "invalid pool part"))?;
    for component in parts {
        check_component(name, component)?;
    }
    Ok(())
}

/// Validates a snapshot name such as `tank/home@daily-2026-11-27`.
pub fn validate_snapshot_name(name: &str) -> Result<()> {
    check_length(name)?;
    let (dataset, snapshot) = name
        .split_once('@')
        .ok_or_else(|| invalid(name, "a snapshot name needs `@`"))?;
    validate_dataset_name(dataset).map_err(|_| invalid(name, "invalid dataset part"))?;
    if snapshot.contains('@') || snapshot.contains('/') {
        return Err(invalid(name, "invalid snapshot part"));
    }
    check_component(name, snapshot)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pool_names() {
        for ok in ["tank", "rpool", "Backup_2", "data-1", "a.b:c"] {
            assert!(validate_pool_name(ok).is_ok(), "{ok}");
        }
        for bad in [
            "", "-o", "1tank", "mirror0", "raidz", "draid2", "spare", "log", "ta nk", "tank/x",
            "tank@s", "t;rm",
        ] {
            assert!(validate_pool_name(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn dataset_names() {
        for ok in [
            "tank",
            "tank/home",
            "tank/home/My Documents",
            "rpool/ROOT/ubuntu_1",
        ] {
            assert!(validate_dataset_name(ok).is_ok(), "{ok}");
        }
        for bad in [
            "tank/",
            "/tank",
            "tank//home",
            "tank/../etc",
            "tank/a@b",
            "tank/a#b",
            "-r/x",
            "tank/a\nb",
            "tank/é",
        ] {
            assert!(validate_dataset_name(bad).is_err(), "{bad:?}");
        }
        assert!(validate_dataset_name(&format!("tank/{}", "a".repeat(300))).is_err());
    }

    #[test]
    fn snapshot_names() {
        assert!(validate_snapshot_name("tank/home@daily-2026-11-27").is_ok());
        assert!(validate_snapshot_name("tank@x").is_ok());
        for bad in ["tank/home", "tank@", "@x", "tank@a@b", "tank@a/b", "-x@y"] {
            assert!(validate_snapshot_name(bad).is_err(), "{bad}");
        }
    }
}
