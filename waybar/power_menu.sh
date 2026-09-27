#!/bin/sh
# Power menu for the waybar custom/power button (anchored dropdown under the button).
# Order: Lock, Suspend, Hibernate, Logout, Reboot, Shutdown (least to most
# destructive). Clicking the button again while the menu is open closes it.
# Hibernate is shown only when the box can really do it (needs a disk-backed
# swap; zram alone cannot hold the image). Actions use
# 'systemctl --no-ask-password' so a polkit denial fails fast and shows a
# notification instead of hanging on an invisible TTY prompt.
# Anchored dropdown menu (no rofi/wofi): waybar-dropdown, see scripts/
DROPDOWN="${DROPDOWN:-$HOME/.local/bin/waybar-dropdown}"
LOCK="${XDG_RUNTIME_DIR:-/tmp}/waybar-power-menu.lock"
# Overridable for testing
POWER_STATE_FILE="${POWER_STATE_FILE:-/sys/power/state}"
SWAPS_FILE="${SWAPS_FILE:-/proc/swaps}"

# Toggle: a second click closes the open menu
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

# Hibernate needs a persistent swap device (not zram) for the image
hibernate=""
if grep -qw disk "$POWER_STATE_FILE" 2>/dev/null &&
   awk 'NR > 1 && $1 !~ /^\/dev\/zram/ { found = 1 } END { exit !found }' "$SWAPS_FILE" 2>/dev/null; then
    hibernate="󰐥 Hibernate\n"
fi

menu="󰌾 Lock\n󰒼 Suspend\n${hibernate}󰌋 Logout\n󰋜 Reboot\n󰆑 Shutdown"

chosen=$(printf '%b' "$menu" | "$DROPDOWN" --anchor-cursor)

fail() {
    notify-send -a power-menu -u critical "Power menu" "$1"
}

case "$chosen" in
    *Lock*)      "$HOME/.local/bin/sway-lock" || fail "Screen lock failed" ;;
    *Suspend*)   systemctl --no-ask-password suspend || fail "Suspend failed" ;;
    *Hibernate*) systemctl --no-ask-password hibernate || fail "Hibernate failed (needs a disk-backed swap)" ;;
    *Logout*)    swaymsg exit ;;
    *Reboot*)    systemctl --no-ask-password reboot || fail "Reboot failed" ;;
    *Shutdown*)  systemctl --no-ask-password poweroff || fail "Shutdown failed" ;;
esac
