# Animations and SwayFX

Reviewed with SwayFX 0.6, based on Sway 1.12. See [sway/config](../sway/config)
for the active settings.

| Setting | Value |
|---------|-------|
| `animation_duration_ms` | `250` |
| `default_dim_inactive` | `0.0` |
| `blur` | `enable` |
| `blur_passes` / `blur_radius` | `5` / `5` |
| `blur_xray` | `disable` |
| `corner_radius` | `0` |
| `smart_corner_radius` | `enable` |
| `shadows` / `shadows_on_csd` | `disable` / `disable` |
| `default_border` | `normal 1` |
| `default_floating_border` | `pixel 1` |
| `hide_edge_borders` | `smart` |
| `titlebar_border_thickness` | `1` |
| `titlebar_padding` | `4 2` |

The final blur/corner/shadow settings follow the system config.d includes.
Game-specific rules disable blur for matching Steam, Proton, gamescope,
Minecraft, Lutris, and Heroic windows, including their windowed views.

SwayNC's 250 ms animation matches SwayFX's duration. Wallpapers are managed by
awww; `swaybg_command -` disables Sway's built-in background launcher. Reloading
the configuration preserves the running awww daemon and its wallpaper state.
