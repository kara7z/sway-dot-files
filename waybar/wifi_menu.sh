#!/bin/sh
LOCK="/tmp/waybar_wifi_menu.lock"

if [ -f "$LOCK" ]; then
    old_pid=$(cat "$LOCK")
    if kill -0 "$old_pid" 2>/dev/null; then
        kill "$old_pid" 2>/dev/null
        rm -f "$LOCK"
        exit 0
    else
        rm -f "$LOCK"
    fi
fi

echo $$ > "$LOCK"
trap 'rm -f "$LOCK"' EXIT

wifi=$(nmcli -t -f active,ssid,signal,security dev wifi 2>/dev/null)
iface=$(nmcli -t -f DEVICE,TYPE device status 2>/dev/null | grep ':wifi$' | cut -d: -f1)

[ -z "$iface" ] && exit 0

current=$(echo "$wifi" | grep '^yes:' | cut -d: -f2)

list=$(echo "$wifi" | grep -v '^$' | sort -t: -k3,3rn | awk -F: '!seen[$2]++' | while IFS=: read -r active ssid signal security; do
    [ -z "$ssid" ] && continue
    sig=$((signal))
    if [ "$sig" -ge 75 ]; then
        icon=""
    elif [ "$sig" -ge 50 ]; then
        icon=""
    elif [ "$sig" -ge 25 ]; then
        icon=""
    else
        icon=""
    fi
    if [ "$active" = "yes" ]; then
        printf "\u2713 %s  %s%%  %s\n" "$icon" "$signal" "$ssid"
    else
        printf "  %s  %s%%  %s\n" "$icon" "$signal" "$ssid"
    fi
done)

[ -z "$list" ] && exit 0

count=$(echo "$list" | grep -c .)

maxlen=$(echo "$list" | awk '{ print length }' | sort -rn | head -1)
width=$((maxlen * 10 + 40))

chosen=$(echo "$list" | wofi --dmenu \
    --width $width --location center \
    --prompt "WiFi" --hide-search \
    --lines $count \
    --conf /dev/null \
    --style /home/kara/.config/wofi/power.css)

[ -z "$chosen" ] && exit 0

ssid=$(echo "$chosen" | sed 's/^.*%  //')

if [ "$ssid" = "$current" ]; then
    action=$(echo -e "Disconnect\nForget\nCancel" | wofi --dmenu \
        --width 300 --location center \
        --prompt "$ssid (Connected)" --hide-search \
        --lines 3 \
        --conf /dev/null \
        --style /home/kara/.config/wofi/power.css)
    [ "$action" = "Disconnect" ] && nmcli device disconnect "$iface"
    [ "$action" = "Forget" ] && nmcli connection delete "$ssid"
    exit 0
fi

security=$(echo "$wifi" | grep -F ":${ssid}:" | head -1 | awk -F: '{print $NF}' | xargs)
saved=$(nmcli -t -f name connection show 2>/dev/null | grep -Fx "$ssid")

if [ -n "$security" ] && [ "$security" != "--" ]; then
    if [ -n "$saved" ]; then
        action=$(echo -e "Connect\nForget\nCancel" | wofi --dmenu \
            --width 300 --location center \
            --prompt "$ssid" --hide-search \
            --lines 3 \
            --conf /dev/null \
            --style /home/kara/.config/wofi/power.css)
        [ "$action" = "Forget" ] && { nmcli connection delete "$ssid"; exit 0; }
        [ "$action" != "Connect" ] && exit 0
    else
        action=$(echo -e "Connect\nCancel" | wofi --dmenu \
            --width 300 --location center \
            --prompt "$ssid" --hide-search \
            --lines 2 \
            --conf /dev/null \
            --style /home/kara/.config/wofi/power.css)
        [ "$action" != "Connect" ] && exit 0
    fi
    result=$(nmcli device wifi connect "$ssid" 2>&1)
    if echo "$result" | grep -q "successfully"; then
        exit 0
    fi
    pass=$(python3 /home/kara/.config/waybar/wifi_pass.py "$ssid" 2>/dev/null)
    [ -n "$pass" ] && nmcli device wifi connect "$ssid" password "$pass"
else
    if [ -n "$saved" ]; then
        action=$(echo -e "Connect\nForget\nCancel" | wofi --dmenu \
            --width 300 --location center \
            --prompt "$ssid" --hide-search \
            --lines 3 \
            --conf /dev/null \
            --style /home/kara/.config/wofi/power.css)
        [ "$action" = "Forget" ] && { nmcli connection delete "$ssid"; exit 0; }
        [ "$action" != "Connect" ] && exit 0
    else
        action=$(echo -e "Connect\nCancel" | wofi --dmenu \
            --width 300 --location center \
            --prompt "$ssid" --hide-search \
            --lines 2 \
            --conf /dev/null \
            --style /home/kara/.config/wofi/power.css)
        [ "$action" != "Connect" ] && exit 0
    fi
    nmcli device wifi connect "$ssid" 2>/dev/null
fi
