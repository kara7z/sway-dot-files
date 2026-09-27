#!/bin/sh
# Portable — uses $HOME (no hardcoded user path)
# Anchored dropdown (no rofi/wofi): waybar-dropdown drops under the clicked module
DROPDOWN="${DROPDOWN:-$HOME/.local/bin/waybar-dropdown}"
LOCK="${XDG_RUNTIME_DIR:-/tmp}/waybar-wifi-menu.lock"

# Small action menu helper: ask <prompt> <lines>  (items on stdin)
ask() {
    "$DROPDOWN" --anchor-cursor --width 300 --prompt "$1" --max-lines "$2"
}

# Masked password prompt (dropdown entry)
ask_password() {
    "$DROPDOWN" --anchor-cursor --password --prompt "Password for $1"
}

# Toggle: a second click on the waybar network icon closes the open menu
if [ -f "$LOCK" ]; then
    old_pid=$(cat "$LOCK" 2>/dev/null)
    if [ -n "$old_pid" ] && kill -0 "$old_pid" 2>/dev/null; then
        case "$(cat "/proc/$old_pid/comm" 2>/dev/null)" in
            sh|bash|dash)
                pkill -P "$old_pid" 2>/dev/null   # the dropdown child
                kill "$old_pid" 2>/dev/null
                rm -f "$LOCK"
                exit 0
                ;;
        esac
    fi
    rm -f "$LOCK"
fi

echo $$ > "$LOCK"
trap 'rm -f "$LOCK"' EXIT

# Fast path: cached AP list (~7 ms). Plain 'dev wifi' rescans whenever the
# cache is older than 30 s and blocks the menu for ~3 s (and briefly disturbs
# the radio, which shows up as ping spikes in games).
wifi=$(nmcli -t -f active,ssid,signal,security dev wifi list --rescan no 2>/dev/null)
# Cold boot / empty cache: pay for one real scan so the menu is never empty
[ -z "$wifi" ] && wifi=$(nmcli -t -f active,ssid,signal,security dev wifi list --rescan yes 2>/dev/null)
iface=$(nmcli -t -f DEVICE,TYPE device status 2>/dev/null | grep ':wifi$' | cut -d: -f1)

[ -z "$iface" ] && exit 0

current=$(printf '%s\n' "$wifi" | grep '^yes:' | cut -d: -f2)

list=$(printf '%s\n' "$wifi" | grep -v '^$' | sort -t: -k1,1r -k3,3rn | awk -F: '!seen[$2]++' | while IFS=: read -r active ssid signal security; do
    [ -z "$ssid" ] && continue
    sig=$((signal))
    if [ "$sig" -ge 75 ]; then
        icon="󰤨"
    elif [ "$sig" -ge 50 ]; then
        icon="󰤥"
    elif [ "$sig" -ge 25 ]; then
        icon="󰤢"
    else
        icon="󰤟"
    fi
    if [ "$active" = "yes" ]; then
        printf "\u2713 %s  %s%%  %s\n" "$icon" "$signal" "$ssid"
    else
        printf "  %s  %s%%  %s\n" "$icon" "$signal" "$ssid"
    fi
done)

[ -z "$list" ] && exit 0

count=$(printf '%s\n' "$list" | grep -c .)

maxlen=$(printf '%s\n' "$list" | awk '{ print length }' | sort -rn | head -1)
width=$((maxlen * 10 + 40))
[ "$width" -gt 640 ] && width=640   # long SSIDs must not push the menu off-screen
lines=$count
[ "$lines" -gt 12 ] && lines=12     # cap the height, scroll the rest

chosen=$(printf '%s\n' "$list" | "$DROPDOWN" --anchor-cursor \
    --width "$width" --prompt "WiFi" --max-lines "$lines")

[ -z "$chosen" ] && exit 0

ssid=$(printf '%s' "$chosen" | sed 's/^.*%  //')
[ -z "$ssid" ] && exit 0

# Currently connected network: Disconnect / Forget / Cancel
if [ "$ssid" = "$current" ]; then
    action=$(printf '%b' "Disconnect\nForget\nCancel" | ask "$ssid (Connected)" 3)
    case "$action" in
        Disconnect) nmcli device disconnect "$iface" &&
                        notify-send -a wifi-menu "Wi-Fi" "Disconnected from $ssid" ;;
        Forget)     nmcli connection delete "$ssid" &&
                        notify-send -a wifi-menu "Wi-Fi" "Forgot $ssid" ;;
    esac
    exit 0
fi

security=$(printf '%s\n' "$wifi" | grep -F ":${ssid}:" | head -1 | awk -F: '{print $NF}' | xargs)
saved=$(nmcli -t -f name connection show 2>/dev/null | grep -Fx "$ssid")

if [ -n "$security" ] && [ "$security" != "--" ]; then
    # Secured network -------------------------------------------------------
    if [ -n "$saved" ]; then
        action=$(printf '%b' "Connect\nForget\nCancel" | ask "$ssid" 3)
        [ "$action" = "Forget" ] && { nmcli connection delete "$ssid"; exit 0; }
        [ "$action" != "Connect" ] && exit 0
        # Stored credentials first (fast path, no prompt)
        if nmcli device wifi connect "$ssid" >/dev/null 2>&1; then
            notify-send -a wifi-menu "Wi-Fi" "Connected to $ssid"
            exit 0
        fi
    else
        action=$(printf '%b' "Connect\nCancel" | ask "$ssid" 2)
        [ "$action" != "Connect" ] && exit 0
    fi

    # Password prompt (dropdown masked entry), retried while it is wrong
    tries=0
    while [ "$tries" -lt 3 ]; do
        pass=$(ask_password "$ssid")
        [ -z "$pass" ] && exit 0
        if nmcli device wifi connect "$ssid" password "$pass" >/dev/null 2>&1; then
            notify-send -a wifi-menu "Wi-Fi" "Connected to $ssid"
            exit 0
        fi
        tries=$((tries + 1))
        [ "$tries" -lt 3 ] && notify-send -a wifi-menu -u normal "Wi-Fi" "Wrong password for $ssid"
    done
    notify-send -a wifi-menu -u critical "Wi-Fi" "Could not connect to $ssid"
    exit 1
else
    # Open network ----------------------------------------------------------
    if [ -n "$saved" ]; then
        action=$(printf '%b' "Connect\nForget\nCancel" | ask "$ssid" 3)
        [ "$action" = "Forget" ] && { nmcli connection delete "$ssid"; exit 0; }
        [ "$action" != "Connect" ] && exit 0
    else
        action=$(printf '%b' "Connect\nCancel" | ask "$ssid" 2)
        [ "$action" != "Connect" ] && exit 0
    fi
    if nmcli device wifi connect "$ssid" >/dev/null 2>&1; then
        notify-send -a wifi-menu "Wi-Fi" "Connected to $ssid"
    else
        notify-send -a wifi-menu -u critical "Wi-Fi" "Could not connect to $ssid"
        exit 1
    fi
fi
