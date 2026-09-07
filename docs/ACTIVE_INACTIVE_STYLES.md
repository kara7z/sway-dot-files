# Active / Inactive Styles — Sway + Waybar + SwayNC

## Sway Windows `sway/config:46`

```
# sway/config:47   border     bg       text      indicator  child_border
client.focused           #2266cc  #000000  #ffffff  #2266cc  #2266cc  ← ACTIVE
client.focused_inactive  #333333  #000000  #888888  #333333  #333333  ← INACTIVE (was focused)
client.unfocused         #333333  #000000  #555555  #333333  #333333  ← INACTIVE
client.urgent            #ff0000  #ff0000  #ffffff  #ff0000  #ff0000  ← URGENT
client.background        #ffffff  (not set, default)
```

- Active: bright blue `#2266cc` border/child_border, white text, black bg — highly visible.
- Inactive: both dark gray `#333333` border, only text distinguishes `#888888` (focused_inactive) vs `#555555` (unfocused).
- Borders `sway/config:42` `normal 1` tiling, `pixel 1` floating, `hide_edge_borders smart:44`, `titlebar 1px padding 4 2:30`

## Waybar Workspaces `waybar/style.css:58`

| State | Selector | Background | Text | Border |
|-------|----------|------------|------|--------|
| **default** | `#workspaces button` | transparent | `#ffffff` | `border-right white 2px, radius 0 8px 0 0` |
| **hover** | `:hover` | `rgba(0,5,255,0.22)` | — | — |
| **active / focused** | `.focused, .active` | `rgba(0,5,255,0.22)` | `rgba(158,158,255,1)` light purple | `inset 0 -3px #ffffff` |
| **urgent** | `.urgent` | `rgba(255,0,0,0.22)` | `red` | — |

Shared `#clock, #battery, #network ... { padding 0 10px; color #fff; border-bottom white 1px; }` `style.css:107`

## Waybar Module States

### battery `style.css:132`
- default `color white`
- **charging/plugged** `139` `bg rgba(0,255,0,0.22)` green translucent
- **critical** `153` `bg #f53c3c` + `animation blink 0.5s steps(12) infinite alternate 156` — `@keyframes 145` `to { bg #fff; color #000 }`

### network `209`
- default `border-left/right white 1px, color white`
- **hover** `210` `rgba(0,0,255,0.22)`
- **disconnected** `220` `bg rgba(255,0,0,0.22)`

### pulseaudio `224`
- **muted** `232` `bg #90b1b1` gray-blue, `color #2a5c45` dark teal, hover `54` `rgba(54,196,255,0.22)`

### backlight `200`
- hover `201` `rgba(255,255,0,0.22)` yellow tint

### power-profiles-daemon `163`
- hover `164` `rgba(255,0,255,0.22)` magenta, per-state `.performance/.balanced/.power-saver` all `color white:171`

### custom/notification `365`
- base `color #fff, bg #000, border 1px white`
- `.notification` `375` `color #fff` + hover `rgba(255,255,255,0.15):382`
- `.none` `386` `opacity 0, min-width 0, padding 0, font-size 0` — **completely hidden**
- `.dnd-notification` `397` `color #888888` muted gray, `bg black`
- `.dnd-none` `403` hidden
- `inhibited` `417` `color #888888, border white`

### custom/power `436`
- hover `437` `bg rgba(255,0,0,0.35) color red`

### mpd `291`
- base `bg #66cc99, color #2a5c45`
- `.disconnected` `296` `bg #f53c3c`, `.stopped` `300` `bg #90b1b1`, `.paused` `304` `bg #51a37a`

### Waybar container `7`
`window#waybar { bg black, border-bottom 1px white, color white, transition bg 0.5s }`
`hidden { opacity 0.2:17 }`

## SwayNC Notifications `swaync/style.css:2`

Variables `2:27`:
```
--cc-bg rgba(0,0,0,0.85)
--noti-border-color #2266cc (blue, matches sway focused)
--noti-bg 0,0,0 (black)
--noti-bg-hover rgb(20,20,20)
--noti-bg-focus rgba(34,102,204,0.3)
--noti-close-bg rgb(51,51,51)
--noti-close-bg-hover rgb(255,0,0)
--text-color rgb(255,255,255)
--text-color-disabled rgb(136,136,136)
--bg-selected rgb(34,102,204)
--border 1px solid var(--noti-border-color)
--border-radius 0px
--notification-shadow 0 1px 3px rgba(0,0,0,0.8)
```

- Animation `50` `slideInRight 250ms cubic-bezier(0.25,0.46,0.45,0.94)` + `slideOutRight` `67`, synced to swayfx `250ms`
- Row `:focus bg var(--noti-bg-focus):71`
- **low** `87` `border #333333`, `text #888888`, **normal** `95` `border #2266cc`, **critical** `99` `border #ff0000 bg #ff0000 color white`
- Close button hover `176` `bg red border #ff0000`
- Control center `188` `bg rgba(0,0,0,0.95) border 1px #2266cc`

## Color Palette Summary

Base monochrome `black #000 / white #fff` with translucent overlays:
- Active blue `rgba(0,5,255,0.22)` + light purple `rgba(158,158,255,1)`
- Hover whites `rgba(255,255,255,0.15-0.22)`
- Green charging `rgba(0,255,0,0.22)`
- Red critical `#f53c3c` / `#ff0000` / `rgba(255,0,0,0.22)`
- Category `#2ecc71` cpu, `#9b59b6` memory, `#f0932b` temp, `#66cc99` mpd
