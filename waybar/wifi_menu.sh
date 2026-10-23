#!/bin/sh
# Portable — uses $HOME (no hardcoded user path)
# Anchored dropdown (no rofi/wofi): waybar-dropdown opens below the top bar,
# centered under the Wi-Fi button (--anchor-x is pixel-measured, see below).
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
# `wifi_menu.sh --list [yes|no|live]` prints the network list and exits.
# Live mode waits for the scan request to be accepted, then reads cached results.
MENU_KIND=wifi                                # what a second click compares against
# The Wi-Fi button lives in the right-hand block at roughly x=1528..1688, center
# x=1608 on a 1920 px output (pixel-measured with grim; the bar itself is right-
# aligned, so this does not move when the SSID text changes). The panel opens
# centered under the button, clamped to the monitor by waybar-dropdown.
WIFI_ANCHOR_X=1608
WIFI_ANCHOR_WIDTH=300                         # same for every submenu/prompt
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
    run_dropdown --anchor-x "$WIFI_ANCHOR_X" --anchor-edge top --margin-top 5 \
        --width "$WIFI_ANCHOR_WIDTH" --prompt "$1" --max-lines "$2" --stay-open --refocus
}

# Masked password prompt (dropdown entry)
ask_password() {
    run_dropdown --anchor-x "$WIFI_ANCHOR_X" --anchor-edge top --margin-top 5 \
        --password --prompt "Password for $1" --stay-open --refocus
}

# --- the network list --------------------------------------------------------
# build_list <rescan: no|yes> — prints "✓ 󰤨  84%  SSID" lines, the active
# network first and then by signal. Runs in the dropdown's background worker.
build_list() {
    # Put the unescaped SSID last: colons/backslashes belong to its name,
    # not to the fixed metadata columns. Keep spaces and percent signs too.
    networks=$(LC_ALL=C nmcli -t -e no -f active,signal,security,ssid dev wifi list --rescan "$1" 2>/dev/null) || return $?
    printf '%s\n' "$networks" \
      | grep -v '^[[:space:]]*$' \
      | sort -t: -k1,1r -k2,2rn \
      | awk -F: '{
          ssid = $0
          sub(/^[^:]*:[^:]*:[^:]*:/, "", ssid)
          if (ssid == "" || seen[ssid]++) next
          signal = $2 + 0
          icon = signal >= 75 ? "󰤨" : signal >= 50 ? "󰤥" : signal >= 25 ? "󰤢" : "󰤟"
          printf "%s %s  %s%%  %s\n", ($1 == "yes" ? "✓" : " "), icon, $2, ssid
        }'
}

# List-only mode runs before the lock on purpose: the dropdown's Refresh row
# calls it as a child of this very script while the lock is held, so taking the
# lock here would make Refresh print nothing and the list would never update.
if [ "$1" = "--list" ]; then
    if [ "${2:-yes}" = live ]; then
        # Wait for NetworkManager to accept the request. With --wait 0, nmcli
        # can exit before the asynchronous request is delivered. The radio scan
        # finishes in the background; subsequent polls pick up all its results.
        # NetworkManager may throttle requests; still read its latest results.
        timeout 4 nmcli --wait 3 dev wifi rescan >/dev/null 2>&1 || :
        build_list no
    else
        build_list "${2:-yes}"
    fi
    exit $?
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
            sh|bash|dash|wifi_menu.sh|power_menu.sh)
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

# Paint immediately, even with no cached networks. All network queries happen
# after opening, in a worker; poll every second while the list is open.
# Automatic width and a 12-row cap let the panel grow and shrink with results.
chosen=$(run_dropdown --anchor-x "$WIFI_ANCHOR_X" \
    --prompt "WiFi" --max-lines 12 \
    --anchor-edge top --margin-top 5 --refresh-cmd "\"$0\" --list live" --cancel \
    --refresh-interval 1 --empty-label "No networks found" \
    --stay-open --refocus </dev/null)

[ -z "$chosen" ] && exit 0

iface=$(nmcli -t -f DEVICE,TYPE device status 2>/dev/null | grep ':wifi$' | cut -d: -f1 | head -1)
[ -z "$iface" ] && exit 0
current=$(LC_ALL=C nmcli -t -e no -f active,ssid dev wifi list --rescan no ifname "$iface" 2>/dev/null | sed -n 's/^yes://p' | head -1)

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

# Match the complete unescaped SSID, including embedded colons/backslashes.
# ENVIRON avoids awk interpreting backslash sequences in a -v assignment.
wifi=$(LC_ALL=C nmcli -t -e no -f security,ssid dev wifi list --rescan no ifname "$iface" 2>/dev/null)
security=$(printf '%s\n' "$wifi" | WIFI_SSID="$ssid" awk -F: 'substr($0, index($0, ":") + 1) == ENVIRON["WIFI_SSID"] { print $1; exit }')
saved=$(nmcli -t -e no -f name connection show 2>/dev/null | grep -Fx -- "$ssid")

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
