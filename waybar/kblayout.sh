#!/bin/sh
POPUP_PID=""
last=$(swaymsg -t get_inputs | python3 -c "
import sys, json
data = json.load(sys.stdin)
for i in data:
    if i.get('type') == 'keyboard':
        name = i.get('xkb_active_layout_name', '')
        if 'Arabic' in name:
            print('AR')
        else:
            print('US')
        break
" 2>/dev/null)
echo "$last"
while true; do
    current=$(swaymsg -t get_inputs | python3 -c "
import sys, json
data = json.load(sys.stdin)
for i in data:
    if i.get('type') == 'keyboard':
        name = i.get('xkb_active_layout_name', '')
        if 'Arabic' in name:
            print('AR')
        else:
            print('US')
        break
" 2>/dev/null)
    if [ "$current" != "$last" ] && [ -n "$current" ]; then
        echo "$current"
        kill "$POPUP_PID" 2>/dev/null
        echo "$current" | wofi --dmenu \
            --width 200 --height 50 --location center \
            --hide-search --conf /dev/null \
            --style /home/kara/.config/wofi/power.css &
        POPUP_PID=$!
        ( sleep 1 && kill "$POPUP_PID" 2>/dev/null ) &
        last="$current"
    fi
    sleep 0.1
done
