#!/bin/sh
set -eu
cd "$(dirname "$0")"
dest="${HOME}/.local"
offline=false
binary_only=false
while [ "$#" -gt 0 ]; do
    case "$1" in
        --prefix)
            [ "$#" -ge 2 ] || { echo '--prefix requires a directory' >&2; exit 2; }
            dest="$2"; shift 2 ;;
        --offline) offline=true; shift ;;
        --binary-only) binary_only=true; shift ;;
        -h|--help)
            echo 'Usage: sh install.sh [--offline] [--prefix DIRECTORY] [--binary-only]'
            exit 0 ;;
        *) echo "Unknown argument: $1" >&2; exit 2 ;;
    esac
done
set -- build --locked --release --target-dir target
if "$offline"; then set -- "$@" --offline; fi
cargo "$@"
mkdir -p "$dest/bin" "$dest/libexec"
# Rename a fresh executable; never overwrite a running executable in place.
install -m 755 target/release/sway-helpers "$dest/libexec/sway-helpers.new"
mv -f "$dest/libexec/sway-helpers.new" "$dest/libexec/sway-helpers"
for name in sway-helpers sway-workspace sway-mirror-workspaces sway-mirror-workspace-daemon sway-gamepad-idle-guard sway-game-freeze-daemon sway-helper-json; do
    ln -sfn ../libexec/sway-helpers "$dest/bin/$name.new"
    mv -Tf "$dest/bin/$name.new" "$dest/bin/$name"
done
if ! "$binary_only"; then
    for name in sway-display-toggle sway-display-mirror-ensure sway-wallpaper-sync; do
        install -m 755 "scripts/$name" "$dest/bin/$name.new"
        mv -f "$dest/bin/$name.new" "$dest/bin/$name"
    done
fi
printf '%s\n' 'Installed Rust Sway helpers. Reload Sway to replace the gamepad guard.'
