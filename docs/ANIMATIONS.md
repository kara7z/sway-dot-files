# Animations & SwayFX

Source: `sway/config:33` — SwayFX `r7135.87754778` (based on sway 1.12.0)

## Core Timing

| Setting | Value | Lines | Notes |
|---------|-------|-------|-------|
| `animation_duration_ms` | `250` | `sway/config:34`, `/etc/sway/config:3` | preserved from upstream, controls workspace / window open/close |
| `default_dim_inactive` | `0.0` | `sway/config:40` | disabled dimming (0 = no dim) |
| `smart_corner_radius` | `enable` | `sway/config:302` | smart rounding even with radius 0 |

## Blur

| Setting | Value | Lines |
|---------|-------|-------|
| `blur` | `enable` | `sway/config:37` + `296` (reaffirmed after `include`) |
| `blur_xray` | `disable` | `sway/config:297` — blur windows only, not wallpaper through |
| `blur_passes` | `5` | `sway/config:38/298` (upstream `4`) |
| `blur_radius` | `5` | `sway/config:39/299` (upstream `4`) |
| `blur_noise` etc. | default | not set, SwayFX defaults |

> Cost: `passes 5 / radius 5` is heavier on Intel UHD (i5-1235U). For battery, try `3/3`.

Comment `sway/config:36`:
> blur fix: enable globally, disable only for fullscreen windows (keep animation 250ms)

Overridden from upstream `blur disable: /etc/sway/config:5`

## Corners & Shadows

| Setting | Value | Lines | Upstream |
|---------|-------|-------|----------|
| `corner_radius` | `0` | `sway/config:35/301` | `15` at `/etc/sway/config:17` |
| `shadows` | `disable` | `sway/config:303` | `disable` |
| `shadows_on_csd` | `disable` | `sway/config:304` | `enable` at `/etc/sway/config:23` |

Comment `sway/config:300`:
> Reduce corner/shadow effects that trigger fullscreen damage bug (keep blur, drop corners/shadows)

- `gaps inner` — not set locally (upstream `15` at `/etc/sway/config:18` not loaded because `~/.config/sway/config` overrides `/etc/sway/config`). Effective gaps = `0`.

## Borders

```
sway/config:42 default_border normal 1
sway/config:43 default_floating_border pixel 1
sway/config:44 hide_edge_borders smart
sway/config:30 titlebar_border_thickness 1
sway/config:31 titlebar_padding 4 2
```

## Workspace Switch Sync

SwayNC uses same `250ms` (`swaync/style.css:50 @keyframes slideInRight`) to match `animation_duration_ms` — comment at `swaync/style.css:49` notes sync.

## Tuning Suggestions

- Battery saver: `blur_passes 3`, `blur_radius 3`, keep `animation_duration_ms 250`
- Eye-candy: restore `corner_radius 12` + `gaps inner 10` if you like rounded (requires gaps >0)
