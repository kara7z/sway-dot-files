# Void Linux / SwayFX

This profile snapshots the working Void Linux desktop from 2026-11-05.
The original Arch/CachyOS configuration remains the base profile.

## Install

```sh
git clone https://github.com/kara7z/sway-dot-files.git
cd sway-dot-files
./install-void.sh --check
./install-void.sh --all
```

`install.sh` also detects Void via `/etc/os-release`. Select explicitly with
`--profile void` or `--profile arch`. Copy/symlink modes, timestamped backups,
`--offline`, and `--target-home` work with either profile. `--check` never installs
packages, builds helpers, or changes files. `--offline` applies to Cargo builds;
dependency installation still needs network access.

XBPS package names are in [`deps/xbps.txt`](../deps/xbps.txt). The installer
does not enable services or replace your display manager/session configuration.
Run it as your normal user, not root; sudo is used only for packages.

## Session and services

Start SwayFX as `dbus-run-session sway`, either from a TTY or your display manager.
Enable the Void `dbus` and `elogind` runit services if not already running.
Enable `NetworkManager`, `bluetoothd`, and `power-profiles-daemon` if you use
their corresponding desktop features. Do not run competing network managers:
review existing dhcpcd/wpa_supplicant services before switching to NetworkManager.
Consult the Void Handbook for your service setup; no systemd is required.

`sway-void-startup` imports the session's D-Bus environment, starts PipeWire and
the GNOME polkit agent, and restores the wallpaper. Included PipeWire snippets
start WirePlumber and the PulseAudio-compatible server. Avoid starting duplicate
audio services elsewhere. Power-menu actions use elogind's `loginctl`.

## Wallpaper picker and appearance

Waypaper is not currently in the Void package list. To use the included launcher:

```sh
python3 -m venv --system-site-packages ~/.local/share/venvs/waypaper
~/.local/share/venvs/waypaper/bin/pip install 'waypaper==2.9'
```

Bibata Modern Ice and a complete FiraCode Nerd Font are optional manual font/cursor
installations. The XBPS list provides FiraCode and Nerd Font symbols as fallbacks;
adjust `font_family` in Kitty if necessary. Discord is optional and must be
installed separately; it is not downloaded automatically.

## Local adaptations

- SwayFX effects live in `~/.config/sway/effects.conf`; animations are 250 ms.
- Blur stays enabled for the desktop, with Kitty and game exclusions.
- Kitty uses an opaque background and a normally started shared instance.
  The earlier hidden instance produced gray windows; it is not prestarted.
- Alt+Enter opens a new Kitty window. Super+B opens a new Firefox window via
  its D-Bus remote interface, falling back to `firefox --new-window`.
- Super+E requests a new Nautilus window via D-Bus, not just focus-existing.
- Print Screen opens Flameshot's graphical selection/annotation editor using
  its Grim adapter. No portal changes are needed for this capture path.
- `way-displays` uses Void 1.14's sequence-based v1 configuration schema.
  Display settings open `wdisplays`; review the eDP-1/1920×1200 display snapshot
  in `profiles/void/sway/output.conf` and `profiles/void/way-displays/cfg.yaml`
  before use on different hardware.
- The optional Firefox fast launcher uses Python's system PyGObject/GIO;
  the common Sway/Waybar helpers remain compiled Rust.

After installation, validate with `sway -c ~/.config/sway/config --validate`.
Log out and back in to run new `exec` startup commands. A reload only reruns
`exec_always`. Do not enable the SwayFX effects include in plain Sway.

## Screenshot

![Void Linux SwayFX desktop with Kitty](screenshots/void-desktop.png)

The preview is a clean desktop capture; personal browser tabs, terminal history,
and network details are excluded.
