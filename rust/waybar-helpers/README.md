# Rust Waybar helpers

Native replacements for the Python dropdown, media helper, legacy password
prompt, and Python embedded in the keyboard-layout menu. Built with gtk-rs GTK3
and gtk-layer-shell bindings. The CSS and current top-bar positioning are
preserved; Wi-Fi results refresh immediately and every second while open.

## Commands

- `waybar-dropdown`: stdin list → chosen row on stdout; arrows/Home/End/Enter,
  clickable rows, Escape/cancel, masked password entry with a Connect button,
  focus persistence/refocus, periodic/manual asynchronous refresh, and dry-run JSON.
- `waybar-media`: playerctl metadata → Waybar JSON every second. Supports
  `--once` for a single sample and `--player NAME`. Missing players emit empty
  valid JSON; a hung playerctl is killed after two seconds.
- `waybar-wifi-password SSID`: standalone masked prompt, replacing the unused
  `wifi_pass.py` helper. A cancelled prompt prints nothing.
- `waybar-helpers layouts` / `active-layout`: read Sway input JSON on stdin,
  using the first keyboard with multiple layouts. Used by `kblayout_menu.sh`.

One compiled executable supplies all commands through symlinks. `--help` lists
the dropdown options. `WAYBAR_DROPDOWN_DEBUG=1` enables focus/key/refresh logs;
password contents are never logged. A refresh runs in its own process group,
with a ten-second timeout and cancellation when the panel closes. Successful
empty results clear the list; failed refreshes retain the last results.

Layer-shell uses Wayland, with a regular GTK window fallback under X11 for tests.
XQueryPointer supplies cursor positions when available; it is optional. Margins
are relative to the selected monitor, including monitors at nonzero positions.

## Build/install

System build dependencies on Arch/CachyOS: `rust base-devel pkgconf gtk3
gtk-layer-shell libx11`. Runtime needs GTK3, gtk-layer-shell, libX11 and, for
media, playerctl. Python is not required by these runtime helpers.

```sh
cargo test --locked
cargo clippy --locked --all-targets -- -D warnings
cargo fmt -- --check
sh install.sh
```

Add `--offline` when Cargo's cache is populated. The installer accepts
`--prefix DIRECTORY` (default `~/.local`) and atomically replaces the executable
and command links. The main dotfiles installer builds both Rust projects before
replacing configuration and backs up old files, links, and Python entry points.

## GUI validation

From the dotfiles checkout, after building the release executable:

```sh
python3 tests/waybar/runtime.py
```

This optional test harness starts an isolated Sway headless compositor, tests
native layer-shell refresh/password/cancellation behavior, then drives regular
GTK menus through the private XWayland display to check selection, Escape,
Cancel, and Refresh. Test-only dependencies include Python and python-xlib;
these are not installed as Waybar runtime dependencies. The older screenshot
probe is retained in `tests/waybar/anchoring.py` for manual diagnostic use.

Backups from the migration are in `~/.local/state/waybar-rust/20261015/` on the
reviewed machine. The repository contains source and pinned Cargo.lock only;
compiled artifacts are ignored.
