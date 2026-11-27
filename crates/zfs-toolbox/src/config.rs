// SPDX-License-Identifier: GPL-3.0-only
//! Build-time and install-time settings.

use std::path::PathBuf;

/// Application identifier, used by GTK, the `.desktop` file and polkit.
pub const APP_ID: &str = "io.github.gillesmagneville.ZfsToolbox";

/// Name of the gettext translation domain (`zfs-toolbox.mo`).
pub const GETTEXT_PACKAGE: &str = "zfs-toolbox";

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub const REPOSITORY: &str = env!("CARGO_PKG_REPOSITORY");

/// Installation directory of the helper, overridable at build time.
const LIBEXECDIR: &str = match option_env!("ZFS_TOOLBOX_LIBEXECDIR") {
    Some(dir) => dir,
    None => "/usr/libexec/zfs-toolbox",
};

/// Default directory of compiled translations, overridable at build time.
const LOCALEDIR: &str = match option_env!("ZFS_TOOLBOX_LOCALEDIR") {
    Some(dir) => dir,
    None => "/usr/share/locale",
};

/// Path of the privileged helper.
///
/// `ZFS_TOOLBOX_HELPER` overrides it for development. The helper still goes
/// through `pkexec` and polkit, so this grants no extra privilege.
pub fn helper_path() -> PathBuf {
    std::env::var_os("ZFS_TOOLBOX_HELPER")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(LIBEXECDIR).join("zfs-toolbox-helper"))
}

/// Directory holding `<lang>/LC_MESSAGES/zfs-toolbox.mo`.
///
/// In order: `ZFS_TOOLBOX_LOCALEDIR`, then `../share/locale` next to the
/// executable when it holds our translations (AppImage or relocated
/// install), then the build-time default.
pub fn locale_dir() -> PathBuf {
    if let Some(dir) = std::env::var_os("ZFS_TOOLBOX_LOCALEDIR") {
        return PathBuf::from(dir);
    }
    let relocated = std::env::current_exe()
        .ok()
        .and_then(|exe| Some(exe.parent()?.parent()?.join("share/locale")))
        .filter(|dir| {
            std::fs::read_dir(dir).is_ok_and(|langs| {
                langs.flatten().any(|lang| {
                    lang.path()
                        .join("LC_MESSAGES")
                        .join(format!("{GETTEXT_PACKAGE}.mo"))
                        .is_file()
                })
            })
        });
    relocated.unwrap_or_else(|| PathBuf::from(LOCALEDIR))
}
