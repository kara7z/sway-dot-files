#!/usr/bin/env python3
"""Isolated helper regressions: no real GUI, display or network changes."""
import os
from pathlib import Path
import signal
import subprocess
import tempfile
import time
import unittest

REPO = Path(__file__).resolve().parents[1]


class Helpers(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory(prefix="sway-helper-test-")
        self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name)
        self.bin = self.root / ".local/bin"
        self.bin.mkdir(parents=True)
        self.env = dict(os.environ, HOME=str(self.root), TEST_ROOT=str(self.root),
                        XDG_RUNTIME_DIR=str(self.root), XDG_CACHE_HOME=str(self.root),
                        SWAYSOCK="fixture", MENU_LOCK=str(self.root / "menu.lock"),
                        PATH=f"{self.bin}:{os.environ['PATH']}")
        for key in ("SWAY_MIRROR_FILL16X9", "SWAY_MIRROR_SCALING"):
            self.env.pop(key, None)

    def stub(self, name, body):
        path = self.bin / name
        path.write_text("#!/bin/sh\n" + body)
        path.chmod(0o755)
        return path

    def run_helper(self, relative, *args):
        return subprocess.run([str(REPO / relative), *args], env=self.env,
                              capture_output=True, text=True, timeout=10, check=True)

    def test_wifi_names_and_exact_security_match(self):
        self.stub("nmcli", r'''
case "$*" in
  *active,signal,security,ssid*)
    printf '%s\n' 'no:99:WPA2:Cafe:Guest' 'yes:40:WPA2:Home\WiFi' \
      'no:30:WPA2:Cafe:Guest' 'no:80:WPA2:  100% wifi  ' ;;
  *DEVICE,TYPE*) echo 'wlan0:wifi' ;;
  *active,ssid*) echo 'yes:Other' ;;
  *security,ssid*) printf '%s\n' '--:Cafe:Guest:Other' 'WPA2:Cafe:Guest' ;;
  *'connection show'*) : ;;
  *'wifi connect'*) printf '%s\n' "$@" > "$TEST_ROOT/connect.args" ;;
  *) exit 2 ;;
esac
''')
        result = self.run_helper("waybar/wifi_menu.sh", "--list", "no")
        rows = result.stdout.splitlines()
        self.assertEqual(len(rows), 3)
        self.assertEqual(rows[0], "✓ 󰤢  40%  Home\\WiFi")
        self.assertEqual(rows[1], "  󰤨  99%  Cafe:Guest")
        self.assertEqual(rows[2], "  󰤨  80%    100% wifi  ")
        dropdown = self.stub("dropdown", r'''
case "$*" in
  *--password*) echo 'secret:test' ;;
  *--refresh-cmd*) echo '  󰤨  99%  Cafe:Guest' ;;
  *) cat >/dev/null; echo Connect ;;
esac
''')
        self.stub("notify-send", "exit 0\n")
        self.env["DROPDOWN"] = str(dropdown)
        self.run_helper("waybar/wifi_menu.sh")
        self.assertEqual((self.root / "connect.args").read_text().splitlines(),
                         ["device", "wifi", "connect", "Cafe:Guest", "password", "secret:test"])

    def test_menu_second_click_and_switch(self):
        dropdown = self.stub("dropdown", "exec sleep 30\n")
        self.env["DROPDOWN"] = str(dropdown)
        for first_name, second_name in [("power", "power"), ("wifi", "wifi"),
                                         ("power", "wifi"), ("wifi", "power")]:
            with self.subTest(first=first_name, second=second_name):
                children = []
                try:
                    def launch(name):
                        child = subprocess.Popen([str(REPO / f"waybar/{name}_menu.sh")],
                                                 env=self.env, start_new_session=True,
                                                 stdout=subprocess.DEVNULL,
                                                 stderr=subprocess.DEVNULL)
                        children.append(child)
                        return child

                    def wait_panel(owner):
                        deadline = time.monotonic() + 4
                        while time.monotonic() < deadline:
                            try:
                                lock = self.root / "menu.lock"
                                if ((lock / "pid").read_text().strip() == str(owner.pid)
                                        and (lock / "panel").read_text().strip()):
                                    return
                            except FileNotFoundError:
                                pass
                            time.sleep(.02)
                        self.fail("menu did not open")

                    first = launch(first_name)
                    wait_panel(first)
                    second = launch(second_name)
                    self.assertEqual(first.wait(timeout=5), 0)
                    if first_name == second_name:
                        self.assertEqual(second.wait(timeout=5), 0)
                    else:
                        wait_panel(second)
                        third = launch(second_name)
                        self.assertEqual(second.wait(timeout=5), 0)
                        self.assertEqual(third.wait(timeout=5), 0)
                    self.assertFalse((self.root / "menu.lock").exists())
                finally:
                    for child in children:
                        try:
                            os.killpg(child.pid, signal.SIGTERM)
                        except ProcessLookupError:
                            pass
                        child.wait(timeout=5)

    def test_mirror_fit_and_daemon_lock(self):
        self.stub("pgrep", '''
case "$*" in
  *way-displays*) exit 0 ;;
  *wl-mirror*) [ "${EXISTING_MIRROR:-0}" = 1 ] ;;
  *) exit 1 ;;
esac
''')
        self.stub("swaymsg", "echo '[]'\n")
        self.stub("notify-send", "exit 0\n")
        self.stub("sway-helper-json", '''
cat >/dev/null
case "$1" in
  outputs-tsv) printf 'eDP-1\\t1\\t0\\t0\\t1920\\t1200\\t1920\\t1200\\t60\\nDP-1\\t1\\t1920\\t0\\t1920\\t1080\\t1920\\t1080\\t60\\n' ;;
  mirror-ok|has-mode) echo yes ;;
  mirror-pair) printf 'eDP-1\\tDP-1\\n' ;;
  mirror-pids)
    if [ "${EXISTING_MIRROR:-0}" = 1 ]; then echo 99999
    elif [ -f "$TEST_ROOT/mirror.pid" ]; then cat "$TEST_ROOT/mirror.pid"; fi ;;
esac
''')
        self.stub("sway-mirror-workspace-daemon", '''
if [ -e /proc/$$/fd/9 ]; then echo inherited; else echo closed; fi > "$TEST_ROOT/daemon-fd"
''')
        self.stub("wl-mirror", '''
printf '%s\\n' "$@" > "$TEST_ROOT/mirror.args"
echo $$ > "$TEST_ROOT/mirror.pid"
exec sleep 30
''')
        for name, fill, existing in [("sway-display-toggle", "0", "0"),
                                     ("sway-display-toggle", "1", "0"),
                                     ("sway-display-mirror-ensure", "0", "0"),
                                     ("sway-display-mirror-ensure", "0", "1")]:
            with self.subTest(helper=name, fill=fill, existing=existing):
                self.env["EXISTING_MIRROR"] = existing
                if fill == "1":
                    self.env["SWAY_MIRROR_FILL16X9"] = fill
                else:
                    self.env.pop("SWAY_MIRROR_FILL16X9", None)
                script = self.root / "mirror-helper"
                script.write_text((REPO / "scripts" / name).read_text().replace(
                    "LOG=/tmp/wl-mirror.log", 'LOG="$TEST_ROOT/mirror.log"'))
                try:
                    subprocess.run(["bash", str(script)], env=self.env, check=True,
                                   capture_output=True, text=True, timeout=10)
                    if existing == "0":
                        args = (self.root / "mirror.args").read_text().splitlines()
                        self.assertEqual(args[args.index("--scaling") + 1],
                                         "cover" if fill == "1" else "fit")
                    self.assertEqual((self.root / "daemon-fd").read_text().strip(), "closed")
                finally:
                    pidfile = self.root / "mirror.pid"
                    if pidfile.exists():
                        try:
                            os.kill(int(pidfile.read_text()), signal.SIGTERM)
                        except ProcessLookupError:
                            pass
                        pidfile.unlink()


if __name__ == "__main__":
    unittest.main(verbosity=2)
