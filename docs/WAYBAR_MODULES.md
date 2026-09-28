# Waybar Modules — Config & Styling

Source: `waybar/config` + `waybar/style.css:439` — Waybar `0.15.0`
(module audit + kblayout rewrite: 2026-09-24)

## Layout

```json
// waybar/config:9
modules-left:   ["sway/workspaces", "sway/mode", "sway/scratchpad"]
modules-center: ["clock"]
modules-right:  ["custom/notification", "tray", "pulseaudio",
                "custom/kblayout", "network", "power-profiles-daemon",
                "backlight", "battery", "custom/power"]
// mpd removed 2026-09-24 — no MPD server on this box, it only rendered "Disconnected ⓘ"
```

Global: `height 30:3`, `spacing 4:7`

## Active Modules (in bar)

### sway/workspaces `waybar/config:30`
- Settings: `disable-scroll true`, `all-outputs true`, `warp-on-scroll false`
- Styling: `style.css:58` — `padding 0 4px`, `border-right white 2px`, `radius 0 8px`
  - hover `58:66` `rgba(0,5,255,0.22)`
  - active/focused `71` `rgba(0,5,255,0.22)` + `inset 0 -3px #fff` + `color rgba(158,158,255,1)` (light purple)
  - urgent `78` `rgba(255,0,0,0.22)` `red`

### sway/mode `waybar/config:44`
- `format: "<span italic>{}</span>"` — orange bg `82` `rgba(255,165,0,0.22)`

### sway/scratchpad `waybar/config:47`
- `format "{icon} {count}"`, `show-empty false`, tooltip `"{app}: {title}"` — style `332` `rgba(0,0,0,0.2)`

### clock `waybar/config:95`
- `format-alt "{:%Y-%m-%d}"`, tooltip calendar — hover `127` `rgba(255,255,255,0.22)`

### custom/notification `waybar/config:197` → swaync
- Exec `swaync-client -swb:212`, click left `swaync-client -t -sw:213`, right DND ` -d -sw`, middle clear ` -C -sw`
- Icons `200`: `󱅫` notification, `󰂜` none, `󰂠` dnd, `󰪓` dnd-none etc.
- Styling `style.css:365` — hidden when `.none` (`opacity 0`, `font-size 0`: `386`), DND muted gray `#888888:397`, inhibited `417`

### tray `waybar/config:87`
- `spacing 10:89` — `.passive dim:273`, `.needs-attention bg #eb4d4b:277`

### mpd `waybar/config:54` — DISABLED 2026-09-24 (no MPD server on this box)
- Not in `modules-right`: `~/.config/mpd` is missing and `mpd.service` is disabled, so the module only ever rendered `Disconnected ⓘ`. Re-add `"mpd",` to `modules-right` after setting MPD up.
- Interval 5s:59, format `stateIcon + artist - album - title (elapsed/total) ⸨pos|len⸩ vol% :55`
- Styling `291` `bg #66cc99` `color #2a5c45`, disconnected red `#f53c3c:296`, stopped `#90b1b1`, paused `#51a37a`

### pulseaudio `waybar/config:160`
- Click `pavucontrol:177`, formats `"{volume}% {icon}":162`, blues `muted "":165`
- Icons `168`: `default ["","",""]`, `headphone "":169`, etc.
- Styling `224`: muted `bg #90b1b1 color #2a5c45:232`

### custom/kblayout `waybar/config:191` — script rewritten 2026-09-24
- Exec `/config/waybar/kblayout.sh:193`: `jq` poll every 0.25s (python3 at 0.1s measured 31ms/tick ≈ 30% of a CPU core; now 3.8ms/tick ≈ 1.5%), single-instance takeover via pidfile (`pkill waybar` can orphan the old copy), `format "{}":192`, click `xkb_switch_layout next:194` — style `361` `border-left white 1px`

### network `waybar/config:150`
- Click `wifi_menu.sh:164` → `waybar-dropdown` opens **above the bar, under the clicked network module** (the bar sits at the bottom; edge auto-detected, XWayland pointer query, clamped to the monitor): cached AP list (`nmcli … --rescan no`, ~7 ms vs ~2.8 s rescan), strength icons `:159` (wifi_strength_1→4, low→high), password prompt + 3 tries + notifications; wifi `"{essid} ({signal}%) {icon}":158`, disconnected `"Disconnected ⚠":163`
- Styling `209`: base `border-left/right white 1px:214`, disconnected `bg rgba(255,0,0,0.22):220`

### power-profiles-daemon `139` — verified working 2026-09-24 (D-Bus service `net.hadess.PowerProfiles`; its binary is not in `$PATH`, that is normal)
- `format "{icon}":140`, icons `144`: `performance ""`, `balanced ""`, `power-saver ""` — hover `163` magenta `rgba(255,0,255,0.22)`

### backlight `115`
- Device auto, `format "{percent}% {icon}":117`, icons moon/sun `["",""]:118`, hover yellow `rgba(255,255,0,0.22):200`
- Scroll up/down `125/126`: `~/.local/bin/sway-brightness up|down` — 2% steps, floor 1% (never blanks the panel)

### battery `123`
- `warning 30:126`, `critical 15:127`, formats `"{capacity}% {icon}":129`, charging `"{capacity}% ":131`
- Styling `132`: charging `bg rgba(0,255,0,0.22):139`, critical `bg #f53c3c` + `@keyframes blink 145` → white

### custom/power `218`
- `format "⏻ ":225`, `on-click power_menu.sh:227` → anchored dropdown (Lock / Suspend / Hibernate-if-disk-swap / Logout / Reboot / Shutdown; clicking it again closes the menu (toggle); wifi and power share one lock so only one menu can ever be open, and clicking the other module's button closes this one and opens that one), hover `442` `bg rgba(255,0,0,0.35) color red`

## Defined but NOT in bar (ready to enable)

| Module | Lines | Format |
|--------|-------|--------|
| `mpd` | 54 | disabled 2026-09-24 — no MPD server on this box (`~/.config/mpd` missing, `mpd.service` disabled); re-add `"mpd",` to `modules-right` once MPD is set up |
| `keyboard-state` | 35 | `"{name} {icon}"`, locked `:40` |
| `idle_inhibitor` | 80 | `/` |
| `cpu` | 100 | `"{usage}% ":101`, style `187` `bg #2ecc71` |
| `memory` | 104 | `"{}% ":105`, `193` `#9b59b6` |
| `temperature` | 107 | `"{temperatureC}°C {icon}":112`, `260` `#f0932b`, critical `#eb4d4b:264` |
| `custom/media` | 179 | exec `mediaplayer.py:188`, `max-length 40:182`, `spotify :184` |

## Fonts

`* { font-family "FiraCode Nerd Font", monospace; font-size 14px; }` `style.css:1` — requires `otf-font-awesome`

## Scripts

- `kblayout.sh` — prints US/AR/FR (`jq` 0.25s poll, was `python3` @0.1s ≈ 30% of a CPU core), `wofi` popup 200×50 centered auto-killed after 1s, kills any previous instance on start
- `wifi_menu.sh` — cached `nmcli dev wifi list --rescan no` (~7 ms; plain `dev wifi` rescans → ~2.8 s), dedup `awk !seen` (connected first), strength icons, `waybar-dropdown` (no rofi/wofi), password retry x3 + `notify-send`, anchored panel centered above the Wi-Fi module (`--anchor-cursor --anchor-edge bottom`), `--stay-open` so a focus loss never closes it, `Refresh`/`Cancel` rows (`--refresh-cmd "$0 --list yes"` repopulates the list in place), one menu at a time via one lock shared with `power_menu.sh`: the same button toggles the panel shut, the other button swaps to that menu (the panel's pid is recorded in the lock so the second click knows what to close; `--list` deliberately bypasses the lock so the Refresh row works while it is held)
- `waybar-dropdown` (`~/.local/bin`) — python3 + GTK3 + `gtk-layer-shell` dropdown next to the clicked bar button (`--anchor-cursor`: XWayland `XQueryPointer` via python-xlib; `--anchor-x N` for tests). Edge auto: click in the lower half → panel **above the bottom bar** (5 px gap — the bar's exclusive zone already lifts the layer edge past it); pixel-verified with grim (cursor 500→span 380..619, 1500→1380..1619, power button 1700→1580..1819, all row 1164). `--anchor-edge`/`--margin-top`/`--margin-bottom` override; 0.6 s map grace so focus races cannot cancel it; dmenu-style stdin→stdout, `--password`, `--dry-run`, `--stay-open`/`--refocus` (losing focus no longer closes it and the keyboard is re-grabbed so Esc keeps working), `--cancel`/`--cancel-label` and `--refresh-cmd`/`--refresh-label` extra rows; `WAYBAR_DROPDOWN_DEBUG=1` prints a focus/key/refresh trace
- `wifi_pass.py:106` — legacy GTK3 dialog, kept for reference (superseded by `waybar-dropdown --password`)
- `power_menu.sh` — anchored dropdown: Lock, Suspend, Hibernate (only with disk-backed swap — zram-only boxes hide it), Logout, Reboot, Shutdown; `systemctl --no-ask-password` + failure notification; one menu at a time (`--stay-open`: only Escape, the Cancel row or a toggle click closes it; same lock as `wifi_menu.sh`, so the other button closes it and opens the power menu)
- Tests (same files in `scripts/`, deployed to `~/.local/bin`): `test-waybar-dropdown` - argument validation, `--dry-run` geometry and the lock paths with a stub dropdown (56 checks, no GUI); `test-waybar-interactive` - real panels driven over XWayland with XTEST: a `--stay-open` menu must survive a focus loss, Esc/Cancel close it, `--refocus` keeps Esc working, `Refresh` repopulates in place; it reads the panel's own `WAYBAR_DROPDOWN_DEBUG=1` trace and reports "skip" (with the evidence) instead of failing when someone else is clicking the visible menu; `test-waybar-anchoring` - grim screenshot diff proving both menus land in the bottom-right corner above the bar (measured right gap 8-9 px, bottom gap 35-50 px on 1920x1200)
- `mediaplayer.py:45` — `playerctl metadata` → JSON `{"text":"artist - title"}` loop 1s
