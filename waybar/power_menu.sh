#!/bin/sh
# Power menu for the waybar custom/power button (anchored dropdown under the button).
# Order: Lock, Suspend, Hibernate, Logout, Reboot, Shutdown (least to most
# destructive), then Cancel.
# One menu at a time for the whole bar: the power and the Wi-Fi menu share one
# lock. Clicking this button again closes the menu (toggle); clicking the other
# button closes this one and opens that one. A menu otherwise closes only on
# Escape or on Cancel (--stay-open disables the usual close-on-focus-loss).
# Hibernate is shown only when the box can really do it (needs a disk-backed
# swap; zram alone cannot hold the image). Actions use
# 'systemctl --no-ask-password' so a polkit denial fails fast and shows a
# notification instead of hanging on an invisible TTY prompt.
# Anchored dropdown menu (no rofi/wofi): waybar-dropdown, see scripts/
MENU_KIND=power                                # what a second click compares against
DROPDOWN="${DROPDOWN:-$HOME/.local/bin/waybar-dropdown}"
# Directory lock, shared with wifi_menu.sh: mkdir is atomic, so two clicks in the
# same millisecond -- or a power click and a Wi-Fi click -- can never both start
# a menu. MENU_LOCK overrides the path (used by the tests).
LOCK="${MENU_LOCK:-${XDG_RUNTIME_DIR:-/tmp}/waybar-menu.lock}"
# Overridable for testing
POWER_STATE_FILE="${POWER_STATE_FILE:-/sys/power/state}"
SWAPS_FILE="${SWAPS_FILE:-/proc/swaps}"

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

# --- one menu at a time: same button toggles, the other button switches --------
# Closing goes through the panel: the launcher that owns it then finishes by
# itself (empty result) and its trap releases the lock -- which is exactly what
# lets the swap below take the lock over. (Deliberately only the recorded panel
# pid is signalled: an action such as `systemctl suspend`, which runs while the
# lock is still held, must never be interrupted.)
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

# Hibernate needs a persistent swap device (not zram) for the image
hibernate=""
if grep -qw disk "$POWER_STATE_FILE" 2>/dev/null &&
   awk 'NR > 1 && $1 !~ /^\/dev\/zram/ { found = 1 } END { exit !found }' "$SWAPS_FILE" 2>/dev/null; then
    hibernate="󰐥 Hibernate\n"
fi

menu="󰌾 Lock\n󰒼 Suspend\n${hibernate}󰌋 Logout\n󰋜 Reboot\n󰆑 Shutdown"

# --stay-open: focus loss / idle never closes it; --cancel adds the on-screen
# Cancel row; --refocus re-claims the keyboard after a focus loss so that
# Escape always reaches the menu.
chosen=$(printf '%b' "$menu" | run_dropdown --anchor-cursor --anchor-align right \
    --anchor-edge bottom --stay-open --refocus --cancel)

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
