#!/usr/bin/env bash
# sway-dot-files installer — CachyOS / Arch (SwayFX)
# Usage:
#   ./install.sh --check        # dry-run, show what would be linked
#   ./install.sh                # copy files (default, safe)
#   ./install.sh --stow         # use GNU stow (symlinks)
#   ./install.sh --copy         # force copy
#   ./install.sh --deps         # install pacman deps from deps/pacman.txt
set -euo pipefail

REPO="$(cd "$(dirname "$0")" && pwd)"
MODE="copy"
DO_DEPS=false

for arg in "$@"; do
  case "$arg" in
    --stow) MODE="stow" ;;
    --copy) MODE="copy" ;;
    --deps) DO_DEPS=true ;;
    --check) MODE="check" ;;
    -h|--help) echo "Usage: $0 [--stow|--copy|--check] [--deps]"; exit 0 ;;
  esac
done

say(){ echo -e "\033[1;34m[install]\033[0m $*"; }
warn(){ echo -e "\033[1;33m[warn]\033[0m $*"; }
die(){ echo -e "\033[1;31m[err]\033[0m $*"; exit 1; }

if $DO_DEPS; then
  say "Installing deps from deps/pacman.txt ..."
  if command -v pacman &>/dev/null; then
    # Official repo packages: stop at "# --- AUR" marker, ignore comments/empty
    pkgs=$(awk '/^# --- AUR/{exit} !/^#/ && !/^$/ {print $1}' "$REPO/deps/pacman.txt" | tr '\n' ' ')
    say "pacman -S --needed $pkgs"
    # --needed avoids reinstall, continue on missing pkg (e.g. bibata name varies)
    sudo pacman -S --needed $pkgs || warn "Some pacman packages missing — check deps/pacman.txt for name variants"
    # AUR helper (yay or paru) for swayfx
    aur_helper=""
    if command -v yay &>/dev/null; then aur_helper="yay"
    elif command -v paru &>/dev/null; then aur_helper="paru"
    fi
    if [[ -n "$aur_helper" ]]; then
      say "AUR ($aur_helper): swayfx-git scenefx-git"
      $aur_helper -S --needed --noconfirm swayfx-git scenefx-git || warn "AUR install failed — install manually: $aur_helper -S swayfx-git scenefx-git"
    else
      warn "No AUR helper (yay/paru) found — manually install swayfx-git + scenefx-git from AUR"
      warn "  git clone https://aur.archlinux.org/yay.git && cd yay && makepkg -si"
    fi
    # Fonts cache
    fc-cache -f 2>/dev/null || true
  else
    warn "pacman not found (non-Arch?) — skipping deps, install manually from deps/pacman.txt"
  fi
fi

backup_if_exists(){
  local dst="$1"
  if [ -e "$dst" ] && [ ! -L "$dst" ]; then
    local bak="${dst}.bak.$(date +%Y%m%d_%H%M%S)"
    say "Backing up $dst -> $bak"
    mv "$dst" "$bak"
  fi
}

install_copy(){
  local src="$1" dst="$2"
  mkdir -p "$(dirname "$dst")"
  backup_if_exists "$dst"
  cp -r "$src" "$dst"
  say "Copied $src -> $dst"
}

install_stow(){
  command -v stow &>/dev/null || die "stow not installed (sudo pacman -S stow)"
  # stow expects repo layout .config/sway -> we have categorized, so use --copy-like via symlinks manually
  warn "--stow not fully implemented for categorized layout, falling back to symlink per file"
  for src in "$REPO/sway/config" "$REPO/sway/outputs.example" "$REPO/waybar/"* "$REPO/swaync/"* "$REPO/wofi/"*; do
    [ -e "$src" ] || continue
    rel="${src#$REPO/}"
    # map repo path to ~/.config path
    case "$rel" in
      sway/config) dst="$HOME/.config/sway/config" ;;
      sway/outputs.example) dst="$HOME/.config/sway/outputs" ;;
      sway/scripts/*) dst="$HOME/.config/sway/scripts/$(basename "$src")" ;;
      sway/status.sh) dst="$HOME/.config/sway/status.sh" ;;
      sway/wifi-menu.sh) dst="$HOME/.config/sway/wifi-menu.sh" ;;
      waybar/*) dst="$HOME/.config/waybar/$(basename "$src")" ;;
      swaync/*) dst="$HOME/.config/swaync/$(basename "$src")" ;;
      wofi/*) dst="$HOME/.config/wofi/$(basename "$src")" ;;
      scripts/*) dst="$HOME/.local/bin/$(basename "$src")" ;;
      *) continue ;;
    esac
    mkdir -p "$(dirname "$dst")"
    backup_if_exists "$dst"
    ln -sf "$src" "$dst"
    say "Linked $src -> $dst"
  done
}

check_mode(){
  say "DRY-RUN — would install:"
  echo "  sway/config              -> ~/.config/sway/config"
  echo "  sway/outputs.example     -> ~/.config/sway/outputs (review before overwrite)"
  echo "  waybar/*                 -> ~/.config/waybar/"
  echo "  swaync/*                 -> ~/.config/swaync/"
  echo "  wofi/*                   -> ~/.config/wofi/"
  echo "  scripts/*                -> ~/.local/bin/"
  echo "Run without --check to apply."
}

if [[ "$MODE" == "check" ]]; then
  check_mode; exit 0
elif [[ "$MODE" == "stow" ]]; then
  install_stow
else
  say "Installing via copy (safe, backups created)..."
  install_copy "$REPO/sway/config" "$HOME/.config/sway/config"
  # do not overwrite real outputs automatically — instruct user
  if [ ! -e "$HOME/.config/sway/outputs" ]; then
    install_copy "$REPO/sway/outputs.example" "$HOME/.config/sway/outputs"
  else
    say "Skipping outputs (exists): $HOME/.config/sway/outputs — compare with sway/outputs.example manually"
  fi
  mkdir -p "$HOME/.config/sway/scripts"
  install_copy "$REPO/sway/scripts/workspace-fade.sh" "$HOME/.config/sway/scripts/workspace-fade.sh"
  install_copy "$REPO/sway/status.sh" "$HOME/.config/sway/status.sh"
  install_copy "$REPO/sway/wifi-menu.sh" "$HOME/.config/sway/wifi-menu.sh"
  mkdir -p "$HOME/.config/waybar"
  for f in "$REPO/waybar/"*; do [ -f "$f" ] && install_copy "$f" "$HOME/.config/waybar/$(basename "$f")"; done
  mkdir -p "$HOME/.config/swaync"
  for f in "$REPO/swaync/"*; do [ -f "$f" ] && install_copy "$f" "$HOME/.config/swaync/$(basename "$f")"; done
  mkdir -p "$HOME/.config/wofi"
  for f in "$REPO/wofi/"*; do [ -f "$f" ] && install_copy "$f" "$HOME/.config/wofi/$(basename "$f")"; done
  mkdir -p "$HOME/.local/bin"
  for f in "$REPO/scripts/"*; do [ -f "$f" ] && install_copy "$f" "$HOME/.local/bin/$(basename "$f")" && chmod +x "$HOME/.local/bin/$(basename "$f")"; done
fi

say "Done. Validate with: sway -c ~/.config/sway/config --validate && waybar --help | head"
say "Reload sway: Mod+Shift+c  | Restart waybar: pkill waybar; waybar &"
