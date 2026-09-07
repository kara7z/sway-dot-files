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
│   ├── kblayout.sh         # US/AR poll + wofi popup
│   ├── wifi_menu.sh        # nmcli + wofi + wifi_pass.py
│   ├── wifi_pass.py        # GTK3 password dialog
│   ├── power_menu.sh       # wofi power menu
│   ├── mediaplayer.py      # playerctl MPRIS → JSON
│   └── power_menu.xml      # legacy GTK menu (unused)
├── swaync/
│   ├── config.json         # 250ms synced to swayfx
│   └── style.css           # black + blue #2266cc
├── wofi/
│   ├── power.css           # rgba(0,0,0,0.6) border white, FiraCode 22px
│   └── power_config        # width 250 height 300
├── scripts/
│   ├── sway-display-toggle # 4K mirror vs extended
│   ├── sway-wl-mirror-toggle # wl-mirror nearest 2x sharp
│   ├── sway-mirror-1080p   # 1080p mirror (perfect waybar)
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
| `XF86Audio*` `XF86MonBrightness*` | pactl/playerctl/brightnessctl | `252/265` |
| `Alt+p` / `Alt+m` | display/mirror toggle | `269/271` |
| `Print` | grim | `280` |
| `3-finger swipe` | workspace prev/next | `100` |

## Animations

`docs/ANIMATIONS.md`: `animation_duration_ms 250` `blur enable passes 5 radius 5` `corner_radius 0` `shadows disable` — preserves 250ms while fixing workspace-switch wallpaper bug via `include` order (`sway/config:291`).

## Styles

`docs/ACTIVE_INACTIVE_STYLES.md`:
- Sway `client.focused #2266cc` blue vs `unfocused #333333` dark gray (`sway/config:48`)
- Waybar active `rgba(0,5,255,0.22)` + `inset 0 -3px #fff` (`style.css:71`), urgent `rgba(255,0,0,0.22)` (`78`), battery critical `blink #f53c3c` (`153`), swaync `slideInRight 250ms` synced.

## Waybar

`docs/WAYBAR_MODULES.md`: left `workspaces/mode/scratchpad`, center `clock`, right `notification/tray/mpd/pulseaudio/kblayout/network/power-profiles/backlight/battery/power` — uses `custom/kblayout` (`kblayout.sh` `swaymsg get_inputs`), `network` (`wifi_menu.sh`), `pulseaudio` (`pavucontrol`).

## Fixes from Backup

- `exec export QT_QPA_PLATFORMTHEME=qt6ct:305` → `systemctl --user set-environment` + `dbus-update-activation-environment` (`sway/config:307`)
- Added `include ~/.config/sway/outputs:291` before system include

## Validate

```sh
sway -c ~/.config/sway/config --validate
python3 -m json.tool ~/.config/waybar/config > /dev/null  # or use config.json
python3 -m py_compile ~/.config/waybar/wifi_pass.py
bash -n ~/.config/sway/status.sh ~/.config/waybar/*.sh
```

License: MIT
