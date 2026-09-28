#!/bin/sh
# Portable — uses $HOME (no hardcoded user path)
# Anchored dropdown (no rofi/wofi): waybar-dropdown drops at the bottom-right
# corner of the screen (no taskbar covering it).
#
# One menu at a time: the Wi-Fi and the power menu share a single lock. Clicking
# the same button again closes the menu (toggle); clicking the other button closes
# this one and opens that one -- so waybar can never put two panels (or a
# duplicate) on screen. A menu otherwise closes only on Escape or on the Cancel
# row (--stay-open turns off the usual close-on-focus-loss behaviour).
#
# The last two rows are "⟳ Refresh" (re-scans and repopulates the list in
# place, without closing the menu) and "✕ Cancel".
#
# `wifi_menu.sh --list [yes|no]` prints the network list and exits; that is what
# the dropdown runs for its Refresh row.
MENU_KIND=wifi                                # what a second click compares against
DROPDOWN="${DROPDOWN:-$HOME/.local/bin/waybar-dropdown}"
# Directory lock, shared with power_menu.sh: mkdir is atomic, so two clicks in
# the same millisecond -- or a Wi-Fi click and a power click -- can never both
# start a menu. MENU_LOCK overrides the path (used by the tests).
LOCK="${MENU_LOCK:-${XDG_RUNTIME_DIR:-/tmp}/waybar-menu.lock}"

# Every panel runs through here so that its pid lands in the lock directory: the
# next click (this script again) reads it and closes that panel. The launcher then
# finishes on its own -- empty result -- and its trap releases the lock.
run_dropdown() {                              # arguments pass straight through
    out="${TMPDIR:-/tmp}/waybar-menu-$$.out"
    "$DROPDOWN" "$@" >"$out" 2>/dev/null &
    panel=$!
    [ -d "$LOCK" ] && echo "$panel" > "$LOCK/panel"
    wait "$panel"
    rc=$?
    [ -f "$out" ] && cat "$out"
    rm -f "$out" "$LOCK/panel"
    return "$rc"
}

# Small action menu helper: ask <prompt> <lines>  (items on stdin)
ask() {
    run_dropdown --anchor-cursor --anchor-align right --anchor-edge bottom \
        --width 300 --prompt "$1" --max-lines "$2" --stay-open --refocus
}

# Masked password prompt (dropdown entry)
ask_password() {
    run_dropdown --anchor-cursor --anchor-align right --anchor-edge bottom \
        --password --prompt "Password for $1" --stay-open --refocus
}

# --- the network list --------------------------------------------------------
# build_list <rescan: no|yes> — prints "✓ 󰤨  84%  SSID" lines, the active
# network first and then by signal. Used at startup (cached, ~7 ms) and by the
# dropdown's Refresh row (real scan, ~2 s, run off the UI thread there).
build_list() {
    nmcli -t -f active,ssid,signal,security dev wifi list --rescan "$1" 2>/dev/null \
      | grep -v '^[[:space:]]*$' \
      | sort -t: -k1,1r -k3,3rn \
      | awk -F: '!seen[$2]++ && $2 != ""' \
      | while IFS=: read -r active ssid signal security; do
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
      done
}

# List-only mode runs before the lock on purpose: the dropdown's Refresh row
# calls it as a child of this very script while the lock is held, so taking the
# lock here would make Refresh print nothing and the list would never update.
if [ "$1" = "--list" ]; then
    build_list "${2:-yes}"
    exit 0
fi

# --- one menu at a time: same button toggles, the other button switches --------
# Closing goes through the panel: the launcher that owns it then finishes by
# itself (empty result) and its trap releases the lock -- which is exactly what
# lets the swap below take the lock over.
close_running_menu() {
    i=0
    while [ -d "$LOCK" ] && [ "$i" -lt 60 ]; do       # give up after ~3 s
        panel=$(cat "$LOCK/panel" 2>/dev/null)
        [ -n "$panel" ] && kill -TERM "$panel" 2>/dev/null
        sleep 0.05
        i=$((i+1))
    done
}

if ! mkdir "$LOCK" 2>/dev/null; then
    old_pid=$(cat "$LOCK/pid" 2>/dev/null)
    old_kind=$(cat "$LOCK/kind" 2>/dev/null)
    if [ -n "$old_pid" ] && kill -0 "$old_pid" 2>/dev/null; then
        # Still our own launcher (comm check guards against PID reuse)
        case "$(cat "/proc/$old_pid/comm" 2>/dev/null)" in
            sh|bash|dash)
                close_running_menu
                # same button: that click meant "close it", so we are done
                [ "$old_kind" = "$MENU_KIND" ] && exit 0
                ;;
        esac
    else
        rm -rf "$LOCK"                       # stale lock (launcher died)
    fi
    mkdir "$LOCK" 2>/dev/null || exit 0      # other menu was open: it is gone now
fi
echo $$ > "$LOCK/pid"
echo "$MENU_KIND" > "$LOCK/kind"             # which menu owns it (diagnostics)
# Take the panel down with us (Esc, Cancel, a crash, or sway going away)
trap 'panel=$(cat "$LOCK/panel" 2>/dev/null); [ -n "$panel" ] && kill -TERM "$panel" 2>/dev/null; rm -f "$LOCK/pid" "$LOCK/kind" "$LOCK/panel"; rmdir "$LOCK" 2>/dev/null' EXIT INT TERM

# Cached AP list first (~7 ms): 'nmcli ... --rescan yes' would block the menu
# for ~3 s and briefly disturbs the radio, which shows up as ping spikes.
list=$(build_list no)
# Cold boot / stale cache (fewer than 2 networks, or a single cached AP is
# almost always a leftover): pay for one real scan so the menu is never empty.
count=$(printf '%s\n' "$list" | grep -c .)
if [ "$count" -le 1 ]; then
    list=$(build_list yes)
    count=$(printf '%s\n' "$list" | grep -c .)
fi

iface=$(nmcli -t -f DEVICE,TYPE device status 2>/dev/null | grep ':wifi$' | cut -d: -f1)
[ -z "$iface" ] && exit 0
[ "$count" -eq 0 ] && exit 0

current=$(nmcli -t -f active,ssid dev wifi 2>/dev/null | grep '^yes:' | cut -d: -f2)

maxlen=$(printf '%s\n' "$list" | awk '{ print length }' | sort -rn | head -1)
width=$((maxlen * 10 + 40))
[ "$width" -gt 640 ] && width=640   # long SSIDs must not push the menu off-screen
lines=$count
[ "$lines" -gt 12 ] && lines=12     # cap the list height, scroll the rest
                                     # (Refresh + Cancel are always visible)

# --refresh-cmd re-runs this script in --list mode: the Refresh row re-scans
# and repopulates the panel in place, no close/reopen flash.
chosen=$(printf '%s\n' "$list" | run_dropdown --anchor-cursor --anchor-align right \
    --width "$width" --prompt "WiFi" --max-lines "$lines" \
    --anchor-edge bottom --refresh-cmd "$0 --list yes" --cancel \
    --stay-open --refocus)

[ -z "$chosen" ] && exit 0

# Rows look like "✓ 󰤨  84%  MySSID" — the SSID starts after the first run of
# spaces that follows the signal percentage (an SSID may itself contain '%').
ssid=$(printf '%s' "$chosen" | sed 's/^[^%]*%  //')
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
