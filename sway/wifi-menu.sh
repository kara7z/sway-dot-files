#!/bin/bash
# Legacy wofi Wi-Fi menu. The active menu is waybar/wifi_menu.sh (anchored
# dropdown); this one is kept working for a plain swaybar / status.sh setup.
# Secured networks prompt for the password (wofi --password), retried up to 3x.
#
# A bare sway session has no NetworkManager SecretAgent, so
# `nmcli device wifi connect <ssid>` on its own fails with "no secrets";
# passing the password on the command line is what makes this work.

nmcli device wifi rescan >/dev/null 2>&1

wifi=$(nmcli -t -f SSID,SIGNAL,SECURITY device wifi 2>/dev/null)

choice=$(printf '%s\n' "$wifi" |
  awk -F: '$1 != "" {print $1 " (" $2 "%)"}' |
  sort -u |
  wofi --dmenu --prompt "Wi-Fi")

[ -z "$choice" ] && exit 0

ssid="${choice% (*}"

security=$(printf '%s\n' "$wifi" | awk -F: -v s="$ssid" '$1 == s {print $3; exit}')
saved=$(nmcli -t -f name connection show 2>/dev/null | grep -Fx "$ssid")

connected() { notify-send -a wifi-menu "Wi-Fi" "Connected to $ssid"; exit 0; }
failed()    { notify-send -a wifi-menu -u critical "Wi-Fi" "Could not connect to $ssid"; exit 1; }

if [ -n "$security" ] && [ "$security" != "--" ]; then
    # Secured network: stored credentials first (fast path, no prompt)
    if [ -n "$saved" ] && nmcli device wifi connect "$ssid" >/dev/null 2>&1; then
        connected
    fi
    # Masked password prompt, retried while it is wrong
    tries=0
    while [ "$tries" -lt 3 ]; do
        pass=$(wofi --dmenu --password --prompt "Password for $ssid")
        [ -z "$pass" ] && exit 0
        if nmcli device wifi connect "$ssid" password "$pass" >/dev/null 2>&1; then
            connected
        fi
        tries=$((tries + 1))
        [ "$tries" -lt 3 ] && notify-send -a wifi-menu -u normal "Wi-Fi" "Wrong password for $ssid"
    done
    failed
else
    # Open network
    if nmcli device wifi connect "$ssid" >/dev/null 2>&1; then
        connected
    else
        failed
    fi
fi
