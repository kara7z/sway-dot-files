#!/usr/bin/env bash
# Exercise real builds and both installation modes without touching the session.
set -euo pipefail
repo=$(cd "$(dirname "$0")/.." && pwd)
sandbox=$(mktemp -d /tmp/sway-dotfiles-test.XXXXXX)
trap 'rm -rf -- "$sandbox"' EXIT
fail() { echo "FAIL: $*" >&2; exit 1; }
run_install() { bash "$repo/install.sh" "$@" >> "$sandbox/install.log" 2>&1 || { cat "$sandbox/install.log"; fail 'installer'; }; }

# --check must win over --all in either order and leave no destination.
run_install --all --check --target-home "$sandbox/check"
run_install --check --all --target-home "$sandbox/check"
[[ ! -e "$sandbox/check" ]] || fail 'dry-run wrote destination'
if bash "$repo/install.sh" --unknown > /dev/null 2>&1; then fail 'unknown option accepted'; fi

# Preserve the target of an old helper symlink, as well as the old link itself.
copy_home="$sandbox/copy home"
mkdir -p "$copy_home/.local/bin" "$copy_home/.config/sway"
printf 'old shared executable\n' > "$sandbox/old-binary"
ln -s "$sandbox/old-binary" "$copy_home/.local/bin/sway-workspace"
printf 'existing output rules\n' > "$copy_home/.config/sway/outputs"
mkdir -p "$copy_home/.config/waybar"
printf 'old Python media helper\n' > "$copy_home/.config/waybar/mediaplayer.py"
run_install --copy --offline --target-home "$copy_home"
[[ $(cat "$sandbox/old-binary") == 'old shared executable' ]] || fail 'followed old helper symlink'
compgen -G "$copy_home/.local/bin/sway-workspace.bak.*" > /dev/null || fail 'missing old helper backup'
[[ $(cat "$copy_home/.config/sway/outputs") == 'existing output rules' ]] || fail 'outputs overwritten'
for file in sway/config sway/workspaces way-displays/cfg.yaml; do
  cmp "$repo/$file" "$copy_home/.config/$file" || fail "missing or changed $file"
done
for name in sway-helpers sway-workspace sway-mirror-workspaces sway-mirror-workspace-daemon sway-gamepad-idle-guard sway-game-freeze-daemon sway-helper-json; do
  [[ -x "$copy_home/.local/bin/$name" && -L "$copy_home/.local/bin/$name" ]] || fail "missing Rust entry point $name"
  [[ $(readlink "$copy_home/.local/bin/$name") == ../libexec/sway-helpers ]] || fail "wrong Rust link $name"
done
"$copy_home/.local/bin/sway-gamepad-idle-guard" --version
for name in waybar-dropdown waybar-media waybar-wifi-password waybar-helpers; do
  [[ -x "$copy_home/.local/bin/$name" && -L "$copy_home/.local/bin/$name" ]] || fail "missing Rust Waybar command $name"
done
[[ ! -e "$copy_home/.config/waybar/mediaplayer.py" ]] || fail 'old Python media helper retained'
compgen -G "$copy_home/.config/waybar/mediaplayer.py.bak.*" > /dev/null || fail 'missing old media backup'
printf 'Alpha\nBeta\n' | "$copy_home/.local/bin/waybar-dropdown" --dry-run | jq -e '.items == ["Alpha", "Beta"]' >/dev/null
[[ $(printf '[]' | "$copy_home/.local/bin/sway-helper-json" outputs-tsv) == '' ]] || fail 'JSON helper'
# Source copies of the embedded-Python replacements must stay synchronized.
for name in sway-display-toggle sway-display-mirror-ensure sway-wallpaper-sync; do
  cmp "$repo/scripts/$name" "$repo/rust/sway-helpers/scripts/$name" || fail "divergent $name"
done
if rg -n -i 'python|i3ipc' "$repo/sway" "$repo/scripts/"sway-* "$repo/scripts/workspace-swipe"; then
  fail 'Python remains in Sway helpers'
fi
if rg -n '^[^#]*python3|\.py\b' "$repo/waybar"; then fail 'Python remains in Waybar runtime'; fi

link_home="$sandbox/link home"
run_install --stow --offline --target-home "$link_home"
for file in sway/config sway/workspaces sway/scripts/workspace-fade.sh way-displays/cfg.yaml waybar/config; do
  [[ -L "$link_home/.config/$file" ]] || fail "missing config link $file"
  cmp "$repo/$file" "$link_home/.config/$file" || fail "wrong config link $file"
done
for name in sway-display-toggle sway-lock workspace-swipe; do
  [[ -x "$link_home/.local/bin/$name" && -L "$link_home/.local/bin/$name" ]] || fail "missing script link $name"
done
[[ -x "$link_home/.local/bin/waybar-dropdown" ]] || fail 'missing Rust dropdown in symlink mode'
# Copying after a symlink installation must replace links rather than edit sources.
run_install --copy --offline --target-home "$link_home"
[[ ! -L "$link_home/.config/sway/config" ]] || fail 'copy mode retained config symlink'
cmp "$repo/sway/config" "$link_home/.config/sway/config"
compgen -G "$link_home/.config/sway/config.bak.*" > /dev/null || fail 'missing config symlink backup'
"$link_home/.local/bin/sway-gamepad-idle-guard" --version
printf '%s\n' 'PASS: dry-run, copy, symlink, upgrade, backups, and Rust entry points'
