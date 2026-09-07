# Apps — Sway Keybindings & Waybar

## Sway `sway/config:105` — 30 `exec` bindings

### Variables `sway/config:19`
| Var | Value |
|-----|-------|
| `$term` | `kitty:19` |
| `$browser` | `firefox:21` |
| `$DisplayManager` | `nwg-displays:23` |
| `$menu` | `wmenu-run:24` |
| `$Files` | `nautilus:25` |
| `$Bluetooth` | `blueman-applet:27` |

### `bindsym ... exec` (28)

| Binding | Command | App |
|---------|---------|-----|
| `$mod+Return:110` | `exec $term` | kitty |
| `$mod2+d:111` | `exec nwg-displays` | nwg-displays |
| `$mod2+b:112` | `exec firefox` | firefox |
| `$mod2+w:113` | `exec waypaper` | waypaper |
| `$mod2+e:114` | `exec nautilus` | nautilus |
| `$mod+d:118` | `exec wmenu-run` | wmenu |
| `$mod+Shift+r:120` | `pkill waybar && waybar &` | waybar |
| `$mod2+space:132` | `swaymsg input type:keyboard xkb_switch_layout next` | swaymsg |
| `$mod+Shift+e:135` | `swaynag -m ... 'swaymsg exit'` | swaynag |
| `XF86AudioMute:252` | `pactl set-sink-mute @DEFAULT_SINK@ toggle` | pactl (pulse/pipewire) |
| `XF86AudioLowerVolume:253` | `pactl set-sink-volume ... -5%` | pactl |
| `XF86AudioRaiseVolume:254` | `pactl set-sink-volume ... +5%` | pactl |
| `XF86AudioMicMute:255` | `pactl set-source-mute @DEFAULT_SOURCE@ toggle` | pactl |
| `XF86AudioPlay/Pause:258` | `playerctl play-pause` | playerctl |
| `XF86AudioPrev:260` | `playerctl previous` | playerctl |
| `XF86AudioNext:261` | `playerctl next` | playerctl |
| `XF86AudioStop:262` | `playerctl stop` | playerctl |
| `XF86MonBrightnessDown:265` | `brightnessctl set 5%-` | brightnessctl |
| `XF86MonBrightnessUp:266` | `brightnessctl set 5%+` | brightnessctl |
| `$mod+p / Shift+p:269` | `~/.local/bin/sway-display-toggle` | custom script |
| `$mod+m / Shift+m:271` | `~/.local/bin/sway-wl-mirror-toggle` | custom |
| `XF86Display:274` | `sway-wl-mirror-toggle` |  |
| `XF86DisplayToggle:275` | `sway-wl-mirror-toggle` |  |
| `XF86Fn_F6/8:276` | `nwg-displays` |  |
| `Print:280` | `grim` | grim |
| `swipe:3:left:100` | `~/.local/bin/workspace-swipe next` | custom |
| `swipe:3:right:101` | `workspace-swipe prev` |  |

### Startup `exec` (`sway/config:98`)
| Line | App |
|------|-----|
| `98` `exec_always bash -c 'pkill awww-daemon; sleep 0.2; awww-daemon'` | awww-daemon (wallpaper) |
| `99` `exec --no-startup-id blueman-applet` | blueman-applet |
| `286` `exec waybar` | waybar |
| `287` `exec swaync` | swaync |
| `307` `exec systemctl --user set-environment QT_QPA_PLATFORMTHEME=qt6ct` | qt6ct |

### Scripts in `scripts/`
| Script | Purpose |
|--------|---------|
| `sway-display-toggle` | toggle 4K mirror (scale 2.0) vs extended (HDMI 3840x2160 pos 1920) + waybar restart + notify |
| `sway-wl-mirror-toggle` | `wl-mirror --scaling nearest --fullscreen-output HDMI-A-1 eDP-1` 4K sharp mirror |
| `sway-mirror-1080p` | mirror 1080p (downscale, perfect waybar) |
| `workspace-swipe` | `current=$(swaymsg -t get_workspaces | jq ...)`, +1/-1 |
| `sway/scripts/workspace-fade.sh` | fake fade `dpms off / workspace N / dpms on` |
| `sway/status.sh` | legacy swaybar JSON (wifi `nmcli`, vol `pactl`, bat `/sys/.../BAT0`) — **inactive**, uses waybar now |
| `sway/wifi-menu.sh` | `nmcli rescan | wofi` — legacy, replaced by `waybar/wifi_menu.sh` |

## Waybar Modules & Their Apps `waybar/config:9`

| Module | App / Command | Lines |
|--------|---------------|-------|
| `sway/workspaces` | sway IPC | 30 |
| `sway/mode` | sway | 44 |
| `sway/scratchpad` | sway | 47 |
| `clock` | swaybar clock | 95 |
| `custom/notification` | `swaync-client -swb -t -d -C` | 197 |
| `tray` | `blueman` etc. tray icons | 87 |
| `mpd` | `mpd` + `playerctl` icons | 54 |
| `pulseaudio` | `pavucontrol` click | 160 |
| `custom/kblayout` | `swaymsg get_inputs` via `kblayout.sh` | 191 |
| `network` | `nmcli` via `wifi_menu.sh` | 150 |
| `power-profiles-daemon` | `powerprofilesctl` | 139 |
| `backlight` | `brightnessctl` | 115 |
| `battery` | `/sys/class/power_supply/BAT0` | 123 |
| `custom/power` | `power_menu.sh` → `wofi` + `shutdown/reboot/systemctl` | 218 |
| *(unused)* `cpu` `memory` `temperature` | `cpu/memory/temp` | 100/104/107 |
| *(unused)* `custom/media` | `mediaplayer.py` → `playerctl` | 179 |

## Dependency Checklist

From `deps/pacman.txt` + manual:
`swayfx-git scenefx-git waybar swaync swaybg swayidle swaylock wofi wmenu grim wl-mirror awww kitty nautilus nwg-displays waypaper blueman brightnessctl playerctl pavucontrol pipewire-pulse networkmanager python-gobject jq wl-clipboard libnotify power-profiles-daemon`
Fonts: `ttf-firacode-nerd otf-font-awesome`, cursor `Bibata-Modern-Ice:29`
