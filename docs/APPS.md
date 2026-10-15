# Sway applications and helpers

The current keybinding table is in [README.md](../README.md). The complete
configuration is [sway/config](../sway/config); `$mod` is Alt and `$mod2` is Super.

## Startup

Sway starts Waybar, SwayNC, way-displays, Blueman, Discord, the authentication
agent, battery notifications, and Nautilus's background service at login.
Reloads refresh theme settings and the single-instance idle/gamepad helpers.
The wallpaper daemon starts only if its IPC is unavailable, preserving wallpaper
state through reloads. The optional game-freeze daemon is disabled at startup.

way-displays uses [cfg.yaml](../way-displays/cfg.yaml) to manage output modes and
positions. Its callbacks synchronize wallpaper and ensure mirroring, unless the
user has turned mirroring off. The old nwg-displays outputs include is disabled.

## Sway helpers

| Command | Implementation | Purpose |
|---------|----------------|---------|
| `sway-display-toggle` | Shell + Rust JSON helper | Toggle actual wl-mirror mirroring; serialize with hotplug callbacks |
| `sway-display-mirror-ensure` | Shell + Rust JSON helper | Start the mirror at login/hotplug, respecting manual-off state |
| `sway-wallpaper-sync` | Shell + Rust JSON helper | Apply the current wallpaper to outputs missing an image |
| `sway-workspace` | Rust | Execute workspace switch/move while preserving named workspace numbering |
| `sway-mirror-workspaces` | Rust | Keep the mirror on its dedicated workspace and normal windows on the main output |
| `sway-mirror-workspace-daemon` | Rust | Enforce the mirror layout on Sway events; no focus changes for stable layouts |
| `sway-gamepad-idle-guard` | Rust | Read evdev activity; acquire/release inhibitors without stealing existing ones |
| `sway-game-freeze-daemon` | Rust, optional | Heuristic fullscreen presentation recovery; disabled at startup |
| `sway-helper-json` | Rust | Parse output modes, tree/mirror state, and wallpaper paths |
| `sway-game-unstick` | Shell | Manual recovery preserving ordinary/global fullscreen; Super+g |
| `sway-idle-lock` | Shell | One swayidle: lock after 5 min, outputs off after 10 min, lock before sleep |
| `sway-lock` | Shell | Current-wallpaper lock with a single-instance guard; supports `--dry-run` |
| `sway-brightness` | Shell | Brightness changes in 2% steps, bounded to 1%–100% |
| `workspace-swipe` | Shell | Previous/next numbered workspace; skip unnumbered workspaces |
| `sway-battery-notify` | Shell | Low/critical battery notifications |
| `sway-wl-mirror-toggle`, `sway-mirror-1080p` | Shell | Compatibility aliases for the display toggle |

The Sway commands are built from [rust/sway-helpers](../rust/sway-helpers), then
installed under their original command names. Waybar's native menu/media
commands are built from [rust/waybar-helpers](../rust/waybar-helpers).

## Waybar and legacy utilities

Waybar uses `swaync-client`, `pavucontrol`, `nmcli`, `powerprofilesctl`,
`brightnessctl`, `playerctl`, and the GTK `waybar-dropdown` helper. See
[WAYBAR_MODULES.md](WAYBAR_MODULES.md) for module details.

`sway/status.sh`, `sway/wifi-menu.sh`, and `sway/scripts/workspace-fade.sh` are
legacy files, installed for compatibility and not enabled by the main config.
The legacy workspace fade briefly powers outputs off and should not be used as
a replacement for native SwayFX animations.

Dependencies are listed in [deps/pacman.txt](../deps/pacman.txt). The installer
builds Rust helpers before replacing configuration and never launches applications.
