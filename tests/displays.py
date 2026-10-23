#!/usr/bin/env python3
"""Real wl-mirror/workspace tests on three isolated headless Sway outputs."""
import json
import os
from pathlib import Path
import shutil
import signal
import subprocess
import tempfile
import time

repo = Path(__file__).resolve().parents[1]
binary = repo / "rust/sway-helpers/target/debug/sway-helpers"
with tempfile.TemporaryDirectory(prefix="sway-display-test-") as tmp:
    root = Path(tmp)
    bindir = root / ".local/bin"
    bindir.mkdir(parents=True)
    env = dict(os.environ, HOME=tmp, XDG_CONFIG_HOME=str(root / ".config"),
               XDG_RUNTIME_DIR=tmp, XDG_CACHE_HOME=str(root / ".cache"),
               WLR_BACKENDS="headless", WLR_HEADLESS_OUTPUTS="3",
               WLR_LIBINPUT_NO_DEVICES="1",
               PATH=f"{bindir}:{os.environ['PATH']}", SWAY_MIRROR_BACKENDS="screencopy-shm")
    for name in ("SWAYSOCK", "I3SOCK", "WAYLAND_DISPLAY", "DISPLAY",
                 "SWAY_MIRROR_SCALING", "SWAY_MIRROR_FILL16X9", "SWAY_MIRROR_WS",
                 "LIBGL_ALWAYS_SOFTWARE"):
        env.pop(name, None)
    for name in ("sway-display-toggle", "sway-display-mirror-ensure"):
        shutil.copy2(repo / "scripts" / name, bindir / name)
    for name in ("sway-helper-json", "sway-workspace", "sway-mirror-workspaces",
                 "sway-mirror-workspace-daemon"):
        (bindir / name).symlink_to(binary)
    # Keep the real display manager and desktop notifications entirely outside
    # the test. All Sway IPC and wl-mirror calls use the private compositor.
    for name in ("notify-send", "pgrep", "way-displays"):
        path = bindir / name
        path.write_text("#!/bin/sh\nexit 0\n")
        path.chmod(0o755)
    config = root / "config"
    config.write_text('''
output HEADLESS-1 mode 1920x1200 position -1536 -100 scale 1.25
output HEADLESS-2 mode 1280x720 position 0 0
output HEADLESS-3 mode 1280x720 position 1280 0
seat seat0 fallback true
focus_follows_mouse no
xwayland disable
for_window [app_id="^at[.]yrlf[.]wl_mirror$"] border none, inhibit_idle none
no_focus [app_id="^at[.]yrlf[.]wl_mirror$"]
''')
    with (root / "sway.log").open("w") as log:
        compositor = subprocess.Popen(["sway", "-c", str(config)], env=env,
                                      stdout=log, stderr=log)
        try:
            for _ in range(150):
                sockets = list(root.glob("sway-ipc.*.sock"))
                displays = [p for p in root.glob("wayland-*") if p.is_socket()]
                if sockets and displays:
                    break
                if compositor.poll() is not None:
                    raise RuntimeError((root / "sway.log").read_text())
                time.sleep(.05)
            else:
                raise RuntimeError("headless Sway did not start")
            env.update(SWAYSOCK=str(sockets[0]), WAYLAND_DISPLAY=displays[0].name)

            def ipc(kind):
                return json.loads(subprocess.check_output(
                    ["swaymsg", "-r", "-t", kind], env=env, text=True))

            def command(text):
                result = json.loads(subprocess.check_output(["swaymsg", "-r", text], env=env, text=True))
                assert all(item["success"] for item in result), result

            def helper(name, *args, code=0):
                result = subprocess.run([str(bindir / name), *args], env=env,
                                        text=True, capture_output=True, timeout=20)
                assert result.returncode == code, (name, result.stdout, result.stderr,
                                                   (root / "wl-mirror.log").read_text()
                                                   if (root / "wl-mirror.log").exists() else "")
                return result

            def mirrors():
                found = []

                def walk(node, output=""):
                    if node.get("type") == "output":
                        output = node["name"]
                    if node.get("app_id") == "at.yrlf.wl_mirror":
                        found.append((output, node))
                    for child in node.get("nodes", []) + node.get("floating_nodes", []):
                        walk(child, output)

                walk(ipc("get_tree"))
                return found

            def expect_mirror(source, target):
                deadline = time.monotonic() + 4
                while time.monotonic() < deadline:
                    views = mirrors()
                    if (len(views) == 1 and views[0][0] == target
                            and views[0][1]["fullscreen_mode"] == 1
                            and views[0][1]["name"] == f"Sway mirror: {source} -> {target}"):
                        return views[0][1]
                    time.sleep(.05)
                raise AssertionError(views)

            helper("sway-display-toggle", "--main", code=2)
            helper("sway-display-toggle", "--main", "HEADLESS-1", "--ext", "HEADLESS-1", code=2)
            command("output HEADLESS-2 disable; output HEADLESS-3 disable")
            auto_target = next(o["name"] for o in ipc("get_outputs") if o["name"] != "HEADLESS-1")
            helper("sway-display-toggle", "--software-only")
            expect_mirror("HEADLESS-1", auto_target)
            print("PASS: automatically enables a connected disabled external", flush=True)
            helper("sway-display-toggle", "--off")
            assert not mirrors()
            helper("sway-display-mirror-ensure")
            assert not mirrors(), "hotplug ignored manual-off flag"

            command("output HEADLESS-2 enable; output HEADLESS-3 enable")
            helper("sway-display-toggle", "--software-only", "--main", "HEADLESS-2", "--ext", "HEADLESS-3")
            original = expect_mirror("HEADLESS-2", "HEADLESS-3")
            helper("sway-workspace", "workspace", "number", "7")
            time.sleep(.4)
            assert any(w["name"] == "7" and w["output"] == "HEADLESS-2" and w["focused"]
                       for w in ipc("get_workspaces"))
            helper("sway-display-mirror-ensure")
            assert expect_mirror("HEADLESS-2", "HEADLESS-3")["pid"] == original["pid"]
            print("PASS: explicit source/destination, workspace switching, idempotent hotplug", flush=True)

            helper("sway-mirror-workspace-daemon", "--stop")
            command(f'[con_id={original["id"]}] move container to output HEADLESS-2')
            helper("sway-display-mirror-ensure")
            assert expect_mirror("HEADLESS-2", "HEADLESS-3")["pid"] != original["pid"]
            print("PASS: a surviving mirror on the wrong display is recreated", flush=True)

            command("output HEADLESS-1 disable; output HEADLESS-3 disable")
            helper("sway-display-mirror-ensure")
            assert not mirrors(), "stale mirror covers the only remaining screen"
            command("output HEADLESS-3 enable")
            helper("sway-display-mirror-ensure")
            expect_mirror("HEADLESS-2", "HEADLESS-3")
            print("PASS: disconnect removes stale mirror; reconnect restores it", flush=True)

            command("output HEADLESS-2 disable")
            helper("sway-display-mirror-ensure")
            assert not mirrors(), "mirror survives after its capture source is disabled"
            helper("sway-display-toggle", "--off")
            helper("sway-display-mirror-ensure")
            assert not mirrors()
            print("PASS: source/lid-close cleanup and explicit off with one display", flush=True)
        finally:
            if "SWAYSOCK" in env:
                subprocess.run([str(bindir / "sway-mirror-workspace-daemon"), "--stop"],
                               env=env, capture_output=True, timeout=8)
                for _, view in mirrors():
                    try:
                        os.kill(view["pid"], signal.SIGTERM)
                    except ProcessLookupError:
                        pass
            compositor.terminate()
            try:
                compositor.wait(timeout=5)
            except subprocess.TimeoutExpired:
                compositor.kill()
                compositor.wait()
