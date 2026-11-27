#!/bin/sh
# SPDX-License-Identifier: GPL-3.0-only
#
# Translation tasks for ZFS Toolbox.
#
#   scripts/i18n.sh update       Extract strings into po/zfs-toolbox.pot and
#                                merge them into every po/<lang>.po.
#   scripts/i18n.sh check        Fail if a string is missing or fuzzy in any
#                                language (used by the CI).
#   scripts/i18n.sh build [DIR]  Compile translations and translated data
#                                files into DIR (default: target/data).
#
# Requires GNU gettext >= 0.19.8.

set -eu
cd "$(dirname "$0")/.."

DOMAIN=zfs-toolbox
APP_ID=io.github.gillesmagneville.ZfsToolbox
# Use the ITS rules shipped in po/its, not those of the build machine.
GETTEXTDATADIRS="$PWD/po"
export GETTEXTDATADIRS

sources() {
    grep -v -e '^#' -e '^$' po/POTFILES.in | grep "$1" || true
}

# extract OUTPUT.pot
extract() {
    out=$1
    xtmp=$(mktemp -d)
    # Rust is extracted with the C parser: keep translatable strings on a
    # single line (no "\" line continuation) and use plain string literals.
    # shellcheck disable=SC2046
    xgettext --from-code=UTF-8 --language=C --add-comments=Translators: 2>/dev/null \
        --keyword= --keyword=gettext --keyword=ngettext:1,2 --keyword=gettext_f \
        --package-name="$DOMAIN" --msgid-bugs-address="https://github.com/gillesmagneville/zfs-toolbox/issues" \
        -o "$xtmp/rs.pot" $(sources '\.rs$')
    # shellcheck disable=SC2046
    xgettext --language=Desktop --keyword= --keyword=Comment --keyword=Keywords \
        --add-comments=Translators: -o "$xtmp/desktop.pot" $(sources '\.desktop\.in$')
    # shellcheck disable=SC2046
    xgettext --its="$PWD/po/its/metainfo.its" -o "$xtmp/metainfo.pot" $(sources '\.metainfo\.xml\.in$')
    # shellcheck disable=SC2046
    xgettext --its="$PWD/po/its/polkit.its" -o "$xtmp/policy.pot" $(sources '\.policy\.in$')
    msgcat --use-first --sort-by-file -o "$out" "$xtmp/rs.pot" "$xtmp/desktop.pot" "$xtmp/metainfo.pot" "$xtmp/policy.pot"
    rm -rf "$xtmp"
}

update() {
    extract "po/$DOMAIN.pot"
    for lang in $(cat po/LINGUAS); do
        if [ -f "po/$lang.po" ]; then
            msgmerge --quiet --update --backup=none --no-fuzzy-matching "po/$lang.po" "po/$DOMAIN.pot"
        else
            msginit --no-translator --locale="$lang" -i "po/$DOMAIN.pot" -o "po/$lang.po"
        fi
    done
}

check() {
    tmp=$(mktemp -d)
    extract "$tmp/$DOMAIN.pot"
    status=0
    # Compare the strings only, not their line numbers.
    msgcat --no-location --sort-output -o "$tmp/old.pot" "po/$DOMAIN.pot"
    msgcat --no-location --sort-output -o "$tmp/new.pot" "$tmp/$DOMAIN.pot"
    if ! diff -q -I '^"POT-Creation-Date' "$tmp/old.pot" "$tmp/new.pot" >/dev/null; then
        echo "po/$DOMAIN.pot is out of date: run scripts/i18n.sh update" >&2
        status=1
    fi
    for lang in $(cat po/LINGUAS); do
        msgmerge --quiet --no-fuzzy-matching -o "$tmp/$lang.po" "po/$lang.po" "$tmp/$DOMAIN.pot"
        msgfmt --check -o /dev/null "$tmp/$lang.po"
        untranslated=$(msgattrib --untranslated --no-obsolete "$tmp/$lang.po" | grep -c '^msgid ' || true)
        fuzzy=$(msgattrib --only-fuzzy --no-obsolete "$tmp/$lang.po" | grep -c '^msgid ' || true)
        # Each non-empty msgattrib output starts with the header entry.
        [ "$untranslated" -gt 0 ] && untranslated=$((untranslated - 1))
        [ "$fuzzy" -gt 0 ] && fuzzy=$((fuzzy - 1))
        echo "$lang: $untranslated untranslated, $fuzzy fuzzy"
        if [ "$untranslated" -ne 0 ] || [ "$fuzzy" -ne 0 ]; then
            msgattrib --untranslated --only-fuzzy --no-obsolete "$tmp/$lang.po" >&2 || true
            status=1
        fi
    done
    rm -rf "$tmp"
    return $status
}

build() {
    out=${1:-target/data}
    mkdir -p "$out"
    for lang in $(cat po/LINGUAS); do
        mkdir -p "$out/locale/$lang/LC_MESSAGES"
        msgfmt --check -o "$out/locale/$lang/LC_MESSAGES/$DOMAIN.mo" "po/$lang.po"
    done
    msgfmt --desktop -d po --template "data/$APP_ID.desktop.in" -o "$out/$APP_ID.desktop"
    msgfmt --xml -d po --template "data/$APP_ID.metainfo.xml.in" -o "$out/$APP_ID.metainfo.xml"
    msgfmt --xml -d po --template "data/$APP_ID.policy.in" -o "$out/$APP_ID.policy"
    cp "data/icons/$APP_ID.svg" "$out/$APP_ID.svg"
    echo "Translated data written to $out"
}

case "${1:-}" in
    update) update ;;
    check) check ;;
    build) build "${2:-}" ;;
    *) sed -n '3,14p' "$0" >&2; exit 2 ;;
esac
