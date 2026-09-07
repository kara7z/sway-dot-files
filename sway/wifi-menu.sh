#!/bin/bash

nmcli device wifi rescan >/dev/null 2>&1

wifi=$(nmcli -t -f SSID,SIGNAL device wifi |
  awk -F: '$1 != "" {print $1 " (" $2 "%)"}' |
  sort -u |
  wofi --dmenu --prompt "Wi-Fi")

[ -z "$wifi" ] && exit

ssid="${wifi% (*}"

nmcli device wifi connect "$ssid"
