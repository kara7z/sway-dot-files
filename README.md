# sway-dot-files

Backup of **SwayFX** + **Waybar** + **SwayNC** + **Wofi** for CachyOS/Arch — kitty, firefox, nautilus, nwg-displays.

> **Host:** kara-pc, CachyOS rolling, swayfx `r7135.87754778`, waybar `0.15.0`, swaync `0.12.6` — Laptop `eDP-1 1920x1080` + `HDMI-A-1 3840x2160`

## Screenshots

Add to `screenshots/` (not tracked yet): `grim ~/Pictures/$(date +%Y%m%d_%H%M).png`

## Structure

```
sway-dot-files/
├── sway/
│   ├── config              # fixed: include outputs, Qt env via systemd
│   ├── outputs.example     # sanitized template (copy to outputs)
│   ├── scripts/workspace-fade.sh
│   ├── status.sh           # legacy swaybar (inactive)
│   └── wifi-menu.sh        # legacy (use waybar/wifi_menu.sh)
├── waybar/
│   ├── config              # JSONC (// comments, Waybar supports JSONC)
│   ├── style.css           # black/white + translucent overlays
│   ├── kblayout.sh         # US/AR/FR poll (jq, 0.25s) + wofi popup
│   ├── wifi_menu.sh        # nmcli (cached) + waybar-dropdown (anchored)
│   ├── wifi_pass.py        # legacy GTK3 dialog (replaced by dropdown --password)
│   ├── power_menu.sh       # waybar-dropdown power menu (hibernate auto-hidden)
│   ├── mediaplayer.py      # playerctl MPRIS → JSON
│   └── power_menu.xml      # legacy GTK menu (unused)
├── swaync/
│   ├── config.json         # 250ms synced to swayfx
│   └── style.css           # black + blue #2266cc
├── wofi/
│   ├── power.css           # rgba(0,0,0,0.6) border white, FiraCode 22px
│   └── power_config        # width 250 height 300
├── scripts/
│   ├── sway-display-toggle # $mod+p: duplicate screen on external (SHM wl-mirror)
│   ├── sway-wl-mirror-toggle # legacy alias -> sway-display-toggle --software-only
│   ├── sway-mirror-1080p   # legacy alias -> sway-display-toggle --software-only
│   ├── sway-idle-lock      # one swayidle: lock 5 min, outputs off 10 min, lock before sleep
│   ├── sway-lock           # lock now: classic wallpaper lock (shared by idle/before-sleep/Super+L)
│   ├── sway-brightness     # Fn keys + waybar scroll: 2% steps, clamped to 1%..100%
│   ├── waybar-dropdown     # anchored dropdown menu (gtk-layer-shell) for power/wifi buttons
│   ├── sway-game-unstick   # unstick a frozen fullscreen game (Super+G: fullscreen off/on)
│   ├── sway-game-freeze-daemon # auto-detect & auto-unstick frozen game frames in background
│   ├── sway-gamepad-idle-guard # no idle lock while a gamepad is used (evdev -> inhibit_idle)
│   └── workspace-swipe     # 3-finger swipe
├── deps/pacman.txt         # pacman -Q list
├── docs/
│   ├── ANIMATIONS.md       # blur 5/5, corner 0, shadows disable, 250ms
│   ├── ACTIVE_INACTIVE_STYLES.md # sway client.* + waybar states
│   ├── WAYBAR_MODULES.md   # 10 active + 5 inactive modules
│   └── APPS.md             # 30 bindsym exec + waybar apps
└── install.sh
```

## Quick Install — One Command (new device)

```sh
git clone https://github.com/kara7z/sway-dot-files.git && cd sway-dot-files && ./install.sh --all
# --all = deps (49 pacman + 4 AUR) + dotfiles in one go
# then reload
swaymsg reload && waybar --version
```

**Other modes:**
```sh
./install.sh --check   # dry-run
./install.sh --deps    # deps only
./install.sh --copy    # dotfiles only (default, backs up to *.bak.*)
./install.sh           # same as --copy
```

**One-liner without git (curl):**
```sh
bash -c "$(curl -fsSL https://raw.githubusercontent.com/kara7z/sway-dot-files/main/install.sh)" -- --all
```

**Outputs:** `sway/outputs.example` is sanitized — copy to `~/.config/sway/outputs` and edit `pos/mode/scale` via `nwg-displays` or manually.

### Stow Alternative

```sh
sudo pacman -S stow
./install.sh --stow  # symlinks (fallback for categorized layout)
```

## Keymap (Alt = $mod, Super = $mod2)

| Bind | Action | Source |
|------|--------|--------|
| `Alt+Return` | kitty | `sway/config:110` |
| `Super+d` | nwg-displays | `111` |
| `Super+b` | firefox | `112` |
| `Super+w` | waypaper | `113` |
| `Super+e` | nautilus | `114` |
| `Alt+d` | wmenu-run | `118` |
| `Alt+Shift+r` | restart waybar | `120` |
| `Alt+h/l` `Alt+Left/Right` | focus | `142` |
| `Alt+Shift+h/j/k/l` | move | `151` |
| `Alt+1..0` / `Alt+Shift+1..0` | workspace switch/move | `164/175` |
| `Alt+b/v` | splith/splitv | `193` |
| `Alt+s/w/e` | stacking/tabbed/toggle | `197` |
| `Alt+f` | fullscreen | `202` |
| `Alt+Shift+space` | floating toggle | `205` |
| `Alt+minus/Shift+minus` | scratchpad show/move | `219` |
| `Alt+r` | resize mode (h/j/k/l) | `227` |
| `Super+space` | xkb switch US/AR | `132` |
| `Super+L` | lock the screen now (`~/.local/bin/sway-lock`: classic wallpaper lock) | `196` |
| `Super+G` | unstick a frozen fullscreen game (toggle fullscreen off/on on the focused window) | `199` |
| `XF86Audio*` `XF86MonBrightness*` | pactl/playerctl/`sway-brightness` (2% step, 1% floor) | `252/265` |
| `Alt+p` (or `XF86Display`) | duplicate the main screen on the connected external (toggle) | `271/276` |
| `Print` | grim | `280` |
| `3-finger swipe` | workspace prev/next | `100` |

## Animations

`docs/ANIMATIONS.md`: `animation_duration_ms 250` `blur enable passes 5 radius 5` `corner_radius 0` `shadows disable` — preserves 250ms while fixing workspace-switch wallpaper bug via `include` order (`sway/config:291`).

## Styles

`docs/ACTIVE_INACTIVE_STYLES.md`:
- Sway `client.focused #2266cc` blue vs `unfocused #333333` dark gray (`sway/config:48`)
- Waybar active `rgba(0,5,255,0.22)` + `inset 0 -3px #fff` (`style.css:71`), urgent `rgba(255,0,0,0.22)` (`78`), battery critical `blink #f53c3c` (`153`), swaync `slideInRight 250ms` synced.

## Waybar

`docs/WAYBAR_MODULES.md`: left `workspaces/mode/scratchpad`, center `clock`, right `notification/tray/pulseaudio/kblayout/network/power-profiles/backlight/battery/power` — uses `custom/kblayout` (`kblayout.sh`, `jq` poll 0.25s), `network` (`wifi_menu.sh`, anchored dropdown), `pulseaudio` (`pavucontrol`); `mpd` is defined but disabled (no MPD server on this box).

## Fixes from Backup

- `exec export QT_QPA_PLATFORMTHEME=qt6ct:305` → `systemctl --user set-environment` + `dbus-update-activation-environment` (`sway/config:307`)
- Added `include ~/.config/sway/outputs:291` before system include

## Idle Lock & Gamepads

`scripts/sway-idle-lock` runs exactly one `swayidle` live: lock after 5 min (`swaylock` + current
wallpaper), outputs off after 10 min, lock before sleep. A gamepad is read by the game straight
from `/dev/input`, so sway counts the session as idle and locked it mid-play; two fixes are in
`sway/config` + `scripts/`:

- `for_window [class="^steam_app_"] inhibit_idle focus` (Steam/Proton, `.exe`, `gamescope`,
  Minecraft, Lutris, Heroic, Steam client/Big Picture) — a view holding sway's native idle
  inhibitor pauses **both** timers while it has focus; `focus` means the lock resumes as soon as
  the game loses focus / closes (5 min later), `before-sleep` locking is untouched.
- `scripts/sway-gamepad-idle-guard` — watches the joystick evdev devices (`input` group) and
  holds/clears the same `inhibit_idle focus` from real pad input, which also covers games whose
  class/app_id is not in the list. Release after 300 s without pad input
  (`GAMEPAD_IDLE_GUARD_RELEASE_AFTER`).

Check the live state with
`~/.local/bin/sway-gamepad-idle-guard --status` (pads, parsed rules, views that actually hold an
inhibitor right now):

```sh
swaymsg -t get_tree | jq -r '.. | objects | select((.idle_inhibitors.user? // "none") != "none")
  | "view \(.id) [\(.idle_inhibitors.user)] \(.name)"'
```

Verified 2026-09-25 with a 3 s `swayidle` inside the live session: timer fires normally, does not
fire while the focused game view holds the inhibitor, fires again once the inhibitor is cleared.

### Fullscreen games: keep frames flowing (SwayFX + XWayland/Proton)

- `scripts/sway-game-freeze-daemon` runs in the background (`exec_always` in `sway/config`), monitoring
  fullscreen games. If controller input is active while screen pixels stay completely frozen for > 2 seconds,
  it automatically toggles fullscreen off/on to unstick SwayFX/XWayland frame presentation immediately.
- `scripts/sway-game-unstick` (bound to `Super+G`) remains available for manual unfreezing anytime.
- New Steam/Proton windows automatically start with per-window SwayFX `blur disable` so fullscreen games
  never pay for the desktop's blur render pipeline.

### Lock screen (Super+L, idle timeout and before-sleep)

`scripts/sway-lock` runs the classic lock screen with stock `swaylock`: current wallpaper
full-screen, then swaylock's default prompt. No built clock/date/hint, and no
`~/.config/swaylock/config` theme (that file is intentionally absent so swaylock uses its
defaults). A flock + `pgrep` guard keeps two callers from ever stacking two lock screens
(the old bug: unlock once, still locked).

Usage: `sway-lock --dry-run`, `sway-lock --print-wallpaper`.
Tune with `LOCK_WALLPAPER=/path.png` (force the lock image).

## Validate

```sh
sway -c ~/.config/sway/config --validate
python3 -m json.tool ~/.config/waybar/config > /dev/null  # or use config.json
python3 -m py_compile ~/.config/waybar/wifi_pass.py ~/.local/bin/waybar-dropdown
bash -n ~/.config/sway/status.sh ~/.config/waybar/*.sh
printf 'A\nB\n' | ~/.local/bin/waybar-dropdown --dry-run  # menu shape without GUI
```

License: MIT
