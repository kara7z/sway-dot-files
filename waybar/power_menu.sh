#!/bin/sh
chosen=$(printf "󰆑 Shutdown\n󰋜 Reboot\n󰐥 Hibernate\n󰒼 Suspend\n󰌋 Logout" | \
    wofi --dmenu \
    --conf /home/kara/.config/wofi/power_config \
    --style /home/kara/.config/wofi/power.css)
case "$chosen" in
    *Shutdown) shutdown now ;;
    *Reboot) reboot ;;
    *Hibernate) systemctl hibernate ;;
    *Suspend) systemctl suspend ;;
    *Logout) swaymsg exit ;;
esac
