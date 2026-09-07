#!/bin/bash

printf '{"version":1,"click_events":true}\n[\n'

update_status() {
  wifi_signal=$(nmcli -t -f IN-USE,SIGNAL device wifi 2>/dev/null | awk -F: '$1=="*"{print $2}')
  wifi_signal="${wifi_signal:-0}"

  if [ "$wifi_signal" -ge 80 ] 2>/dev/null; then
    wifi_icon="󰤨"
  elif [ "$wifi_signal" -ge 60 ] 2>/dev/null; then
    wifi_icon="󰤥"
  elif [ "$wifi_signal" -ge 40 ] 2>/dev/null; then
    wifi_icon="󰤢"
  elif [ "$wifi_signal" -ge 20 ] 2>/dev/null; then
    wifi_icon="󰤟"
  else
    wifi_icon="󰤭"
  fi

  vol=$(pactl get-sink-volume @DEFAULT_SINK@ 2>/dev/null | awk '{print $5}')
  [ -z "$vol" ] && vol="0%"
  muted=$(pactl get-sink-mute @DEFAULT_SINK@ 2>/dev/null | awk '{print $NF}')
  if [ "$muted" = "yes" ]; then
    vol_icon="󰝟"
    vol="Muted"
  else
    vol_num=$(echo "$vol" | tr -d '%')
    if [ "${vol_num:-0}" -ge 66 ] 2>/dev/null; then
      vol_icon="󰕾"
    elif [ "${vol_num:-0}" -ge 33 ] 2>/dev/null; then
      vol_icon="󰖀"
    else
      vol_icon="󰕿"
    fi
  fi

  bat=$(cat /sys/class/power_supply/BAT0/capacity 2>/dev/null)
  bat_status=$(cat /sys/class/power_supply/BAT0/status 2>/dev/null)
  if [ -n "$bat" ]; then
    if [ "$bat_status" = "Charging" ]; then
      bat_icon="󰂄"
    elif [ "$bat" -ge 75 ] 2>/dev/null; then
      bat_icon="󰁹"
    elif [ "$bat" -ge 50 ] 2>/dev/null; then
      bat_icon="󰂀"
    elif [ "$bat" -ge 25 ] 2>/dev/null; then
      bat_icon="󰁾"
    else
      bat_icon="󰁻"
    fi
    bat="${bat_icon} ${bat}%"
  else
    bat=""
  fi

  date_str="󰥔 $(date '+%a %d %b  %H:%M')"
}

print_status() {
  printf '%s[{"name":"wifi","full_text":"%s %s%%"},{"name":"vol","full_text":"%s %s"},{"name":"bat","full_text":"%s"},{"name":"date","full_text":"%s"}]\n' \
    "$1" "$wifi_icon" "$wifi_signal" "$vol_icon" "$vol" "$bat" "$date_str"
}

update_status
print_status ""

while true; do
  read -t 1 event || {
    update_status
    print_status ","
    continue
  }

  case "$event" in
    *\"name\":\"wifi\"*)
      networkmanager_dmenu &
      ;;
  esac
done
