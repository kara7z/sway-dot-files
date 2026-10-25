#!/usr/bin/env bash
# sway-dot-files installer — CachyOS / Arch (SwayFX)
# Usage:
#   ./install.sh --check        # dry-run, show what would be linked
#   ./install.sh                # copy files (default, safe)
#   ./install.sh --all          # ONE COMMAND: install deps + dotfiles (recommended for new device)
#   ./install.sh --stow         # symlink configuration and shell scripts
#   ./install.sh --copy         # force copy
#   ./install.sh --deps         # install pacman deps from deps/pacman.txt
# One-liner for new device:
#   git clone https://github.com/kara7z/sway-dot-files.git && cd sway-dot-files && ./install.sh --all
set -euo pipefail

REPO="$(cd "$(dirname "$0")" && pwd)"
MODE=""
DO_DEPS=false
TARGET_HOME="$HOME"
OFFLINE=false
DRY_RUN=false

while (($#)); do
  case "$1" in
    --all) MODE="copy"; DO_DEPS=true ;;
    --stow) MODE="stow" ;;
    --copy) MODE="copy" ;;
    --deps) DO_DEPS=true ;;
    --check) DRY_RUN=true ;;
    --offline) OFFLINE=true ;;
    --target-home)
      [[ $# -ge 2 && "$2" == /* && "$2" != / ]] || { echo '--target-home requires an absolute home directory' >&2; exit 2; }
      TARGET_HOME="${2%/}"; shift ;;
    -h|--help)
      echo "Usage: $0 [--all|--stow|--copy|--check|--deps] [--offline] [--target-home DIRECTORY]"
      echo "  --all   One command: deps + dotfiles (new device)"
      echo "  --deps  Only deps"
      echo "  --copy  Only dotfiles (default)"
      echo "  --check Dry-run"
      echo "  --stow Symlink files and install compiled Rust helpers"
      echo "  --offline Build using cached Cargo dependencies"
      echo "  --target-home Install into another home directory (no session reload)"
      exit 0
      ;;
    *) echo "Unknown argument: $1" >&2; exit 2 ;;
  esac
  shift
done

if $DRY_RUN; then
  MODE=check
  DO_DEPS=false
elif [[ -z "$MODE" ]]; then
  if $DO_DEPS; then MODE=deps-only; else MODE=copy; fi
fi

say(){ echo -e "\033[1;34m[install]\033[0m $*"; }
warn(){ echo -e "\033[1;33m[warn]\033[0m $*"; }
die(){ echo -e "\033[1;31m[err]\033[0m $*"; exit 1; }

if $DO_DEPS; then
  say "Installing deps from deps/pacman.txt ..."
  if command -v pacman &>/dev/null; then
    aur_fallback=""
    # Official repo packages: until "# --- AUR" marker — install one-by-one to skip missing (CachyOS vs Arch)
    pkgs=$(awk '/^# --- AUR/{exit} !/^#/ && !/^$/ {print $1}' "$REPO/deps/pacman.txt")
    say "pacman official: $pkgs"
    for pkg in $pkgs; do
      if pacman -Si "$pkg" >/dev/null 2>&1; then
        echo " → pacman -S $pkg"
        sudo pacman -S --needed --noconfirm "$pkg" 2>&1 | tail -n 5 || warn "Failed $pkg"
      else
        warn "Not in official repo (maybe AUR/CachyOS): $pkg — will try AUR"
        # defer to AUR handling below; collect for later
        aur_fallback="$aur_fallback $pkg"
      fi
    done
    # AUR packages: after marker + any official fallback
    aur_pkgs=$(awk 'found && !/^#/ && !/^$/ {print $1} /^# --- AUR/{found=1}' "$REPO/deps/pacman.txt" | tr '\n' ' ')
    aur_pkgs="$aur_pkgs $aur_fallback"
    # trim
    aur_pkgs=$(echo "$aur_pkgs" | tr ' ' '\n' | grep -v '^$' | sort -u | tr '\n' ' ')
    if [[ -n "${aur_pkgs// /}" ]]; then
      aur_helper=""
      if command -v yay &>/dev/null; then aur_helper="yay"
      elif command -v paru &>/dev/null; then aur_helper="paru"
      fi
      if [[ -n "$aur_helper" ]]; then
        say "AUR ($aur_helper): $aur_pkgs"
        for apkg in $aur_pkgs; do
          $aur_helper -S --needed --noconfirm "$apkg" 2>&1 | tail -n 5 || warn "AUR $apkg failed — try: $aur_helper -S $apkg"
        done
      else
        warn "No AUR helper (yay/paru) found — manually install AUR pkgs: $aur_pkgs"
        warn "  git clone https://aur.archlinux.org/yay.git && cd yay && makepkg -si"
      fi
    fi
    # Fonts cache
    fc-cache -f 2>/dev/null || true
  else
    warn "pacman not found (non-Arch?) — skipping deps, install manually from deps/pacman.txt"
  fi
fi

backup_if_exists(){
  local dst="$1"
  if [[ -e "$dst" || -L "$dst" ]]; then
    local bak="${dst}.bak.$(date +%Y%m%d_%H%M%S_%N)"
    say "Backing up $dst -> $bak"
    mv -T -- "$dst" "$bak"
  fi
}

install_file(){
  local src="$1" dst="$2"
  mkdir -p "$(dirname "$dst")"
  # Move symlinks too: copying through one could overwrite a shared binary.
  backup_if_exists "$dst"
  if [[ "$MODE" == stow ]]; then
    ln -s -- "$src" "$dst"
    say "Linked $src -> $dst"
  else
    cp -p -- "$src" "$dst"
    say "Copied $src -> $dst"
  fi
}

if [[ "$MODE" == deps-only ]]; then
  say "Dependency installation finished. Use --all to also install dotfiles."
  exit 0
elif [[ "$MODE" == check ]]; then
  say "DRY-RUN: target $TARGET_HOME (no packages, builds, or files changed)"
  echo "  sway/{config,output.conf,workspaces,scripts,status.sh,wifi-menu.sh} -> .config/sway/"
  echo "  sway/outputs -> .config/sway/outputs (only if absent; inactive snapshot)"
  echo "  way-displays/cfg.yaml -> .config/way-displays/cfg.yaml"
  echo "  waybar/*, swaync/*, wofi/*, waypaper/* -> .config/"
  echo "  wallpapers/* -> Pictures/Wallpapers/"
  echo "  scripts/* -> .local/bin/"
  echo "  rust/sway-helpers -> build binary in .local/libexec/ and link commands in .local/bin/"
  echo "  rust/waybar-helpers -> build native GTK dropdown/media/password helpers"
  exit 0
fi

command -v cargo >/dev/null || die "Cargo is required. Install Rust, or run ./install.sh --all."
RUST_PROJECT="$REPO/rust/sway-helpers"
CARGO_ARGS=(build --locked --release --manifest-path "$RUST_PROJECT/Cargo.toml" --target-dir "$RUST_PROJECT/target")
INSTALL_ARGS=(--prefix "$TARGET_HOME/.local" --binary-only)
if $OFFLINE; then
  CARGO_ARGS+=(--offline)
  INSTALL_ARGS+=(--offline)
fi
# Finish compilation before replacing any installed configuration or helper.
cargo "${CARGO_ARGS[@]}"
WAYBAR_PROJECT="$REPO/rust/waybar-helpers"
WAYBAR_CARGO_ARGS=(build --locked --release --manifest-path "$WAYBAR_PROJECT/Cargo.toml" --target-dir "$WAYBAR_PROJECT/target")
WAYBAR_INSTALL_ARGS=(--prefix "$TARGET_HOME/.local")
if $OFFLINE; then
  WAYBAR_CARGO_ARGS+=(--offline)
  WAYBAR_INSTALL_ARGS+=(--offline)
fi
cargo "${WAYBAR_CARGO_ARGS[@]}"

say "Installing in $MODE mode (existing files and links are backed up)..."
for name in config output.conf workspaces status.sh wifi-menu.sh; do
  install_file "$REPO/sway/$name" "$TARGET_HOME/.config/sway/$name"
done
if [[ ! -e "$TARGET_HOME/.config/sway/outputs" && ! -L "$TARGET_HOME/.config/sway/outputs" ]]; then
  install_file "$REPO/sway/outputs" "$TARGET_HOME/.config/sway/outputs"
else
  say "Keeping existing outputs file; way-displays manages active output settings."
fi
for f in "$REPO/sway/scripts/"*; do
  [[ -f "$f" ]] || continue
  install_file "$f" "$TARGET_HOME/.config/sway/scripts/$(basename "$f")"
done
for dir in way-displays waybar swaync wofi waypaper; do
  for f in "$REPO/$dir/"*; do
    [[ -f "$f" ]] || continue
    install_file "$f" "$TARGET_HOME/.config/$dir/$(basename "$f")"
  done
done
for f in "$REPO/wallpapers/"*; do
  [[ -f "$f" ]] || continue
  install_file "$f" "$TARGET_HOME/Pictures/Wallpapers/$(basename "$f")"
done
for f in "$REPO/scripts/"*; do
  [[ -f "$f" ]] || continue
  install_file "$f" "$TARGET_HOME/.local/bin/$(basename "$f")"
done
for name in sway-helpers sway-workspace sway-mirror-workspaces sway-mirror-workspace-daemon sway-gamepad-idle-guard sway-game-freeze-daemon sway-helper-json; do
  backup_if_exists "$TARGET_HOME/.local/bin/$name"
done
backup_if_exists "$TARGET_HOME/.local/libexec/sway-helpers"
sh "$RUST_PROJECT/install.sh" "${INSTALL_ARGS[@]}"
for name in waybar-helpers waybar-dropdown waybar-media waybar-wifi-password; do
  backup_if_exists "$TARGET_HOME/.local/bin/$name"
done
backup_if_exists "$TARGET_HOME/.local/libexec/waybar-helpers"
for name in test-waybar-interactive test-waybar-anchoring; do
  backup_if_exists "$TARGET_HOME/.local/bin/$name"
done
# Retire old Python runtime entry points while preserving them in backups.
for name in mediaplayer.py wifi_pass.py; do
  backup_if_exists "$TARGET_HOME/.config/waybar/$name"
done
sh "$WAYBAR_PROJECT/install.sh" "${WAYBAR_INSTALL_ARGS[@]}"

say "Done. Config: $TARGET_HOME/.config/sway/config"
say "Validate with sway -c <config-path> --validate, then reload Sway with Alt+Shift+c."
