#!/bin/sh
# Portable paths — uses $HOME / XDG_CONFIG_HOME
WOFI_CONF="${XDG_CONFIG_HOME:-$HOME/.config}/wofi/power_config"
WOFI_STYLE="${XDG_CONFIG_HOME:-$HOME/.config}/wofi/power.css"
chosen=$(printf "󰆑 Shutdown\n󰋜 Reboot\n󰐥 Hibernate\n󰒼 Suspend\n󰌋 Logout" | \
    wofi --dmenu \
    --conf "$WOFI_CONF" \
    --style "$WOFI_STYLE")
case "$chosen" in
    *Shutdown) shutdown now ;;
    *Reboot) reboot ;;
    *Hibernate) systemctl hibernate ;;
    *Suspend) systemctl suspend ;;
    *Logout) swaymsg exit ;;
esac
