#!/bin/sh
set -eu
cd "$(dirname "$0")"
dest="${HOME}/.local"
offline=false
while [ "$#" -gt 0 ]; do
    case "$1" in
        --prefix)
            [ "$#" -ge 2 ] || { echo '--prefix requires a directory' >&2; exit 2; }
            dest="$2"; shift 2 ;;
        --offline) offline=true; shift ;;
        -h|--help) echo 'Usage: sh install.sh [--offline] [--prefix DIRECTORY]'; exit 0 ;;
        *) echo "Unknown argument: $1" >&2; exit 2 ;;
    esac
done
set -- build --locked --release --target-dir target
if "$offline"; then set -- "$@" --offline; fi
cargo "$@"
mkdir -p "$dest/bin" "$dest/libexec"
install -m 755 target/release/waybar-helpers "$dest/libexec/waybar-helpers.new"
mv -f "$dest/libexec/waybar-helpers.new" "$dest/libexec/waybar-helpers"
for name in waybar-helpers waybar-dropdown waybar-media waybar-wifi-password; do
    ln -sfn ../libexec/waybar-helpers "$dest/bin/$name.new"
    mv -Tf "$dest/bin/$name.new" "$dest/bin/$name"
done
printf '%s\n' 'Installed Rust Waybar helpers.'
