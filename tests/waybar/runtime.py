#!/usr/bin/env python3
"""GUI regression checks in an isolated headless Sway; never use the live socket.
Python is a test-only dependency. Runtime helpers are compiled Rust.
"""
import json
import os
from pathlib import Path
import signal
import subprocess
import sys
import tempfile
import time

repo = Path(__file__).resolve().parents[2]
binary = Path(os.environ.get('WAYBAR_TEST_BINARY', repo / 'rust/waybar-helpers/target/release/waybar-helpers'))
with tempfile.TemporaryDirectory(prefix='waybar-rust-gui-') as tmp:
    runtime = Path(tmp)
    env = dict(os.environ, XDG_RUNTIME_DIR=tmp, WLR_BACKENDS='headless', WLR_HEADLESS_OUTPUTS='1',
               WLR_LIBINPUT_NO_DEVICES='1', LIBGL_ALWAYS_SOFTWARE='1', WAYBAR_DROPDOWN_DEBUG='1',
               GTK_A11Y='none', NO_AT_BRIDGE='1')
    for key in ('SWAYSOCK', 'I3SOCK', 'WAYLAND_DISPLAY', 'DISPLAY', 'GDK_BACKEND'):
        env.pop(key, None)
    config = runtime / 'config'
    config.write_text('output HEADLESS-1 mode 1920x1200\nseat seat0 fallback true\nxwayland force\nfocus_follows_mouse no\nfont pango:monospace 10\n')
    log = (runtime / 'sway.log').open('w')
    compositor = subprocess.Popen(['sway', '-c', str(config)], env=env, stdout=log, stderr=log)
    children = []
    try:
        for _ in range(150):
            sockets = list(runtime.glob('sway-ipc.*.sock'))
            displays = [p for p in runtime.glob('wayland-*') if p.is_socket()]
            if sockets and displays:
                break
            if compositor.poll() is not None:
                raise RuntimeError((runtime / 'sway.log').read_text())
            time.sleep(.05)
        else:
            raise RuntimeError('headless compositor did not start')
        env.update(SWAYSOCK=str(sockets[0]), WAYLAND_DISPLAY=displays[0].name, GDK_BACKEND='wayland')
        from PIL import Image, ImageChops
        before = runtime / 'before.png'
        subprocess.run(['grim', '-o', 'HEADLESS-1', str(before)], env=env, check=True)
        def launch(args, items=''):
            error = runtime / f'menu-{len(children)}.log'
            child = subprocess.Popen([str(binary), 'dropdown'] + args, env=env, text=True,
                                     stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=error.open('w'))
            child.stdin.write(items)
            child.stdin.close()
            children.append(child)
            return child, error
        # Long lists, empty startup, periodic refresh, and shrinking/growing results.
        counter = runtime / 'counter'
        refresh = runtime / 'refresh.sh'
        refresh.write_text(f'#!/bin/sh\nn=$(cat "{counter}" 2>/dev/null || echo 0)\nn=$((n+1))\necho "$n" > "{counter}"\nif [ "$n" -eq 1 ]; then printf "Alpha\\nBeta\\n"; elif [ "$n" -eq 2 ]; then :; else printf "Gamma\\n"; fi\n')
        child, error = launch(['--prompt', 'WiFi test', '--anchor-x', '1608', '--anchor-edge', 'top',
                               '--margin-top', '5', '--refresh-cmd', f'sh "{refresh}"', '--refresh-interval', '1',
                               '--stay-open', '--refocus', '--cancel'])
        time.sleep(.5)
        during = runtime / 'during.png'
        subprocess.run(['grim', '-o', 'HEADLESS-1', str(during)], env=env, check=True)
        bounds = ImageChops.difference(Image.open(before).convert('RGB'), Image.open(during).convert('RGB')).getbbox()
        assert bounds is not None and 1200 < bounds[0] < 1700 and bounds[1] <= 8 and bounds[2] <= 1920, bounds
        print(f'PASS: native menu rendered at configured top anchor: {bounds}', flush=True)
        time.sleep(3.4)
        assert child.poll() is None, error.read_text()
        trace = error.read_text()
        assert trace.count('refresh done:') >= 3, trace
        assert '0 items, err=None' in trace, trace
        assert 'panicked' not in trace and 'CRITICAL' not in trace, trace
        child.send_signal(signal.SIGTERM)
        assert child.wait(timeout=3) == 1, error.read_text()
        print('PASS: native layer-shell menu, live refresh, empty results, resize, SIGTERM')
        # A slow scan must be cancelled together with descendants when toggled off.
        pidfile = runtime / 'scan.pid'
        child, error = launch(['--refresh-cmd', f'echo $$ > "{pidfile}"; sleep 60', '--refresh-interval', '1', '--stay-open'])
        for _ in range(60):
            if pidfile.exists(): break
            time.sleep(.05)
        assert pidfile.exists(), error.read_text()
        scan_pid = int(pidfile.read_text())
        child.terminate()
        assert child.wait(timeout=3) == 1, error.read_text()
        try:
            os.kill(scan_pid, 0)
        except ProcessLookupError:
            pass
        else:
            raise AssertionError('refresh child survived menu exit')
        print('PASS: closing a menu cancels an active scan process group')
        child, error = launch(['--password', '--prompt', 'Password for test', '--stay-open', '--refocus'])
        time.sleep(.7)
        assert child.poll() is None, error.read_text()
        child.terminate()
        assert child.wait(timeout=3) == 1
        assert not child.stdout.read(), 'cancelled password printed text'
        print('PASS: native masked password panel and empty output on cancellation')
        # Reuse the existing keyboard/click/focus test harness under this compositor's XWayland.
        out = subprocess.check_output(['swaymsg', '-r', '-t', 'get_version'], env=env, text=True)
        xdisplay = json.loads(out).get('xwayland_display')
        if not xdisplay:
            # get_version omits DISPLAY on some Sway versions. Ask Sway to export
            # only its private XWayland DISPLAY into this temporary directory.
            target = runtime / 'display'
            subprocess.run(['swaymsg', 'exec', f'sh -c \'printf %s "$DISPLAY" > "{target}"\''], env=env, check=True, capture_output=True)
            for _ in range(60):
                if target.exists() and target.read_text(): break
                time.sleep(.05)
            xdisplay = target.read_text()
        driver = runtime / 'waybar-dropdown'
        driver.symlink_to(binary)
        xenv = dict(env, DISPLAY=xdisplay, BIN=str(driver), GDK_BACKEND='x11')
        result = subprocess.run([sys.executable, str(repo / 'tests/waybar/interactive.py')], env=xenv, text=True, capture_output=True, timeout=100)
        print(result.stdout)
        assert result.returncode == 0, result.stdout + result.stderr
        assert '0 passed' not in result.stdout, result.stdout
    finally:
        for child in children:
            if child.poll() is None:
                child.terminate()
                try: child.wait(timeout=3)
                except subprocess.TimeoutExpired: child.kill(); child.wait()
        compositor.terminate()
        try: compositor.wait(timeout=4)
        except subprocess.TimeoutExpired: compositor.kill(); compositor.wait()
        log.close()
