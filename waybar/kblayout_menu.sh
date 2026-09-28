#!/bin/sh
# Keyboard layout menu for the waybar custom/kblayout button (anchored dropdown).
# Shows the configured layouts (US/AR/FR) — picking one switches immediately.
# Single instance like the wifi/power menus: same button toggles, other menu
# swaps, Esc/Cancel closes. Also still prints the active layout code for the bar.
MENU_KIND=kblayout                              # what a second click compares against
DROPDOWN="${DROPDOWN:-$HOME/.local/bin/waybar-dropdown}"
# The layout button sits left of the network module at roughly x=1409..1484,
# center x=1446 on a 1920 px output (pixel-measured with grim). The panel opens
# centered under the button, clamped to the monitor by waybar-dropdown.
KBLAYOUT_ANCHOR_X=1446
KBLAYOUT_ANCHOR_WIDTH=300                       # same for every submenu/prompt
# Directory lock, shared with the other menus: mkdir is atomic, so clicks on two
# buttons in the same millisecond cannot both start a menu. MENU_LOCK overrides
# the path (used by the tests).
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

# Layouts come from the keyboard itself: the first device with more than one
# layout wins; fall back to the configured us,ara,fr triple.
layouts=$(swaymsg -t get_inputs 2>/dev/null |
    python3 -c 'import json,sys
try:
    data = json.load(sys.stdin)
except Exception:
    data = []
names = []
for dev in data:
    if dev.get("type") == "keyboard" and len(dev.get("xkb_layout_names") or []) > 1:
        names = dev["xkb_layout_names"]
        break
print("\n".join(names))' 2>/dev/null)
[ -z "$layouts" ] && layouts="English (US)
Arabic
French"

current=$(swaymsg -t get_inputs 2>/dev/null |
    python3 -c 'import json,sys
try:
    data = json.load(sys.stdin)
except Exception:
    data = []
name = ""
for dev in data:
    if dev.get("type") == "keyboard" and len(dev.get("xkb_layout_names") or []) > 1:
        name = dev.get("xkb_active_layout_name") or ""
        break
print(name)' 2>/dev/null)

active="$current"
[ -z "$active" ] && active=$(printf '%s\n' "$layouts" | sed -n '1p')

maxlen=$(printf '%s\n' "$layouts" | awk '{ print length }' | sort -rn | head -1)
width=$((maxlen * 10 + 40))
[ "$width" -gt 640 ] && width=640   # keep the menu on-screen
[ "$width" -lt 300 ] && width=300

# Mark the active layout (with *) so it reads as a status, and sort the active
# one to the top so the most relevant row is one row down from the top.
rows=$(printf '%s\n' "$layouts" | awk -v a="$active" '
    $0 == a { printf "*  %s (active)\n", $0; next }
    { print "   " $0 }')

chosen=$(printf '%s\n' "$rows" | run_dropdown --anchor-x "$KBLAYOUT_ANCHOR_X" \
    --width "$width" --prompt "Keyboard" --max-lines 5 \
    --anchor-edge bottom --stay-open --refocus)

[ -z "$chosen" ] && exit 0
# Strip the markers (row looks like "*  English (US) (active)") back to a layout
# name, then switch by index so the right layout ends up active even if a device
# has a different active one than the first multi-layout keyboard.
target=$(printf '%s' "$chosen" | sed 's/^[* ]*//; s/ (active)$//')
[ -z "$target" ] && exit 0
[ "$target" = "$active" ] && exit 0

index=$(printf '%s\n' "$layouts" | grep -n -x -F "$target" | head -1 | cut -d: -f1)
[ -z "$index" ] && index=$(printf '%s\n' "$layouts" | grep -n -F "$target" | head -1 | cut -d: -f1)
[ -z "$index" ] && exit 0

swaymsg input type:keyboard xkb_switch_layout "$((index - 1))" >/dev/null
notify-send -a layout-menu "Keyboard" "Switched to $target"
