#!/bin/sh
# waybar custom/kblayout — prints the active layout (US/AR/FR) for the bar and
# pops a short notify-send toast whenever the layout changes.
#
# 2026-09-24 fixes:
#   * jq instead of python3 per poll: 31ms -> 3.8ms (the old version burned ~30%
#     of a CPU core doing 10 python3 starts per second)
#   * poll every 0.25s (0.1s was overkill for a layout indicator)
#   * single instance WITH TAKEOVER: `pkill waybar` only signals the `sh -c`
#     wrapper, so the old script was orphaned and kept running — every waybar
#     restart added another popup-spamming copy. The new copy now kills the
#     previous one (verified via /proc/<pid>/cmdline) before taking over.
#
# 2026-09-28: the wofi toast is gone; layout changes now show a notify-send
# toast instead (no wofi/rofi dependency left in the bar).
PIDFILE="${XDG_RUNTIME_DIR:-/tmp}/waybar-kblayout.pid"

old=$(cat "$PIDFILE" 2>/dev/null || true)
if [ -n "$old" ] && [ "$old" != "$$" ] && [ -r "/proc/$old/cmdline" ] &&
   grep -qa 'kblayout.sh' "/proc/$old/cmdline"; then
    kill "$old" 2>/dev/null
fi
printf '%s\n' "$$" > "$PIDFILE"

get_layout() {
    swaymsg -t get_inputs 2>/dev/null |
        jq -r '[.[] | select(.type == "keyboard")][0].xkb_active_layout_name // ""' 2>/dev/null |
        {
            read -r name
            case "$name" in
                *Arabic*) echo AR ;;
                *French*) echo FR ;;
                *) echo US ;;
            esac
        }
}

last=$(get_layout)
echo "$last"
while true; do
    sleep 0.25
    current=$(get_layout)
    [ "$current" = "$last" ] && continue
    last="$current"
    echo "$current"
    notify-send -a layout-menu -t 1000 "Keyboard" "$current"
done
