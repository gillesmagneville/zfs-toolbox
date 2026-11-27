// SPDX-License-Identifier: GPL-3.0-or-later
//! Translations.
//!
//! Source strings are written in English. At start-up gettext picks the
//! catalogue matching the system language (`LANGUAGE`, `LC_ALL`,
//! `LC_MESSAGES`, `LANG`). Languages without a catalogue (anything other
//! than French, German and Spanish) automatically fall back to English.

use gettextrs::{
    bind_textdomain_codeset, bindtextdomain, gettext, setlocale, textdomain, LocaleCategory,
};

use crate::config::{locale_dir, GETTEXT_PACKAGE};

pub fn init() {
    // SAFETY: called first thing in `main`, before any other thread exists
    // and before anything reads the locale.
    unsafe {
        setlocale(LocaleCategory::LcAll, "");
    }
    // Failures only mean that the interface stays in English.
    let _ = bindtextdomain(GETTEXT_PACKAGE, locale_dir());
    let _ = bind_textdomain_codeset(GETTEXT_PACKAGE, "UTF-8");
    let _ = textdomain(GETTEXT_PACKAGE);
}

/// Translates `msgid` and replaces each `{}` with the next argument.
///
/// Translators may not reorder the placeholders; use one placeholder per
/// message when the order could differ between languages.
pub fn gettext_f(msgid: &str, args: &[&str]) -> String {
    let mut text = gettext(msgid);
    for arg in args {
        if let Some(pos) = text.find("{}") {
            text.replace_range(pos..pos + 2, arg);
        }
    }
    text
}

#[cfg(test)]
mod tests {
    use super::gettext_f;

    #[test]
    fn replaces_placeholders_in_order() {
        assert_eq!(gettext_f("{} of {}", &["1", "2"]), "1 of 2");
        assert_eq!(gettext_f("no placeholder", &["x"]), "no placeholder");
    }
}
