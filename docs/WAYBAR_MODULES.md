# Waybar modules and helpers

The live configuration is backed up in [waybar/config](../waybar/config), with
styling in [waybar/style.css](../waybar/style.css). The bar is positioned at the
top. Shell menu launchers preserve their current anchors and five-pixel gap.

| Position | Modules |
|----------|---------|
| Left | Sway workspaces, mode, scratchpad |
| Center | Clock |
| Right | Notifications, tray, audio, keyboard layout, network, power profile, backlight, battery, power |

MPD and custom media are defined but not enabled in the module arrays. The
custom media command now points to `~/.local/bin/waybar-media`, so enabling the
module uses Rust. Its output is JSON, with artist/title text, player icon
selection, status class, and a valid empty record when no player is available.

## Menus

The network, keyboard, and power menus share a directory lock. Clicking the
same button closes its panel; clicking a different button swaps menus. The
`waybar-dropdown` Rust executable provides the GTK3/layer-shell surface.

- Wi-Fi opens immediately, refreshes through NetworkManager every second,
  preserves selection across signal-strength changes, and provides manual
  Refresh/Cancel rows. Passwords use masked entry with a Connect button.
- Keyboard selection parses Sway's multi-layout keyboard JSON through
  `waybar-helpers`, then changes layout silently. The bar indicator retains its
  shell/jq polling helper and US/AR/FR text.
- The power menu retains its existing system actions and hibernate checks.

The dropdown supports arrows, Home/End, Enter, mouse selection, Escape,
`--stay-open`, and `--refocus`. A refresh runs asynchronously so the menu stays
responsive; closing the panel terminates its scan process group. Failed scans
retain results; successful empty scans show the empty-state message.

## Other commands

Notifications use `swaync-client`; audio opens `pavucontrol`. Backlight scroll
calls `sway-brightness`. Media keys use `playerctl`, and power profiles use
`powerprofilesctl`. Existing CSS controls module active/urgent/critical states.

`waybar/mediaplayer` and `waybar/wifi_pass` are compatibility shell launchers
for the Rust commands. The old `.py` runtime files are backed up and retired.
Python remains an optional dependency of GUI test scripts under `tests/waybar/`.

Build/install instructions and command details:
[rust/waybar-helpers/README.md](../rust/waybar-helpers/README.md).
