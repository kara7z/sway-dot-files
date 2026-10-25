# sway-dot-files

SwayFX, Waybar, SwayNC, Wofi, and way-displays configuration for CachyOS/Arch.
The Sway configuration and helper fixes reviewed on 2026-10-15 are included here,
along with the Rust source, pinned dependencies, tests, and installer.
The custom Waybar Python helpers have also been migrated to Rust; see
[the migration results](docs/WAYBAR_RUST_MIGRATION.md).

The reviewed machine uses SwayFX 0.6 (based on Sway 1.12), an eDP-1 laptop panel
at 1920×1200, and a Samsung external display configured for its advertised
1920×1080 at 60 Hz. Adjust display names and modes for another machine.

## Install

From this checkout:

```sh
./install.sh --check         # preview only; never installs packages or builds
./install.sh --copy          # build Rust helpers and copy dotfiles (default)
./install.sh --stow          # symlink dotfiles; install compiled Rust helpers
./install.sh --all           # install Arch/AUR dependencies, then build and copy
./install.sh --deps          # install dependencies only
```

Cargo and a C linker are needed to build the helpers. `deps/pacman.txt` includes
`rust` and `base-devel`; the compositor requires SwayFX. Builds use `Cargo.lock`
and can download missing crates. Add `--offline` to use an existing Cargo cache.
AUR packages require `yay` or `paru`, or manual installation.

Existing files **and symlinks** are renamed to timestamped `.bak.*` siblings
before replacement. Both copy and symlink modes install the workspace include,
way-displays settings, shell scripts, and Rust command entry points. The installer
does not reload Sway or launch programs. `--stow` uses direct symlinks and does not
require GNU Stow; keep this checkout at its current location when using it.

To verify an installation elsewhere without changing the current home directory:

```sh
./install.sh --copy --offline --target-home /tmp/sway-install-review
```

The source remains in `rust/sway-helpers/`. The executable is installed to
`~/.local/libexec/sway-helpers`, with command symlinks in `~/.local/bin/`.
Keep the repository to rebuild or update it.

After reviewing the display settings, validate and reload in your Sway session:

```sh
sway -c ~/.config/sway/config --validate
swaymsg reload
```

A newly installed `exec` startup command runs at the next Sway login; reload only
reruns `exec_always` commands. The configuration uses `way-displays` for display
management. `sway/outputs.example` is an inactive template and an existing outputs
file is preserved. Do not enable its include while way-displays controls layouts.
The 2026-10-25 live snapshot includes `sway/output.conf` (actively included),
the inactive nwg-displays `sway/outputs`, Waypaper settings, and all 27 wallpapers
from `~/Pictures/Wallpapers`. Timestamped old backups are intentionally excluded.
The installer restores these assets and the active output include; existing
`outputs` files are preserved. Review the machine-specific output names/modes
before restoring on another computer. Waypaper's stylesheet path currently
references `/home/kara`; that optional stylesheet does not exist on the source
machine. The referenced optional keybindings file is also absent.

## Files and helpers

| Path | Contents |
|------|----------|
| `sway/` | Main configuration, workspace output preference, legacy bar/menu scripts |
| `way-displays/cfg.yaml` | Display modes, lid handling, wallpaper/mirror callbacks |
| `rust/sway-helpers/` | Rust source, Cargo.lock, unit/integration tests, standalone installer |
| `rust/waybar-helpers/` | Native GTK menus, media output, and keyboard JSON parsing |
| `scripts/` | Shell Sway helpers, workspace swipe, and Waybar dropdown utilities |
| `waybar/`, `swaync/`, `wofi/` | Panel, notification, and menu configuration |
| `waypaper/`, `wallpapers/` | Wallpaper picker settings and wallpaper images |
| `deps/pacman.txt` | Arch/CachyOS package dependencies |
| `docs/SWAY_AUDIT.md` | Fixed bugs, verification results, and hardware testing limits |
| `tests/install.sh` | Isolated installation and upgrade checks |

All Python previously used in the **Sway helpers**, including embedded snippets
in display and wallpaper scripts, has been replaced with Rust. One executable
provides these commands:

- `sway-workspace`
- `sway-mirror-workspaces`
- `sway-mirror-workspace-daemon`
- `sway-gamepad-idle-guard`
- `sway-game-freeze-daemon` (optional, disabled at startup)
- `sway-helper-json`

See [the Rust helper documentation](rust/sway-helpers/README.md) for options.
Waybar's GTK dropdown, password prompt, media output, and embedded keyboard
JSON parsing also use Rust. Shell menu launchers remain in `waybar/`.
Third-party applications retain their own dependencies.

The Waybar executable is installed to `~/.local/libexec/waybar-helpers`, with
`waybar-dropdown`, `waybar-media`, `waybar-wifi-password`, and `waybar-helpers`
entry points in `~/.local/bin`. Its GTK3, gtk-layer-shell, and X11 libraries
come from the package list. Python is optional for GUI test scripts only.

## Behavior and keybindings

`Alt` is `$mod`; `Super` is `$mod2`.

| Binding | Action |
|---------|--------|
| Alt+Return | Kitty |
| Alt+d | Application launcher |
| Super+b / Super+e / Super+w | Firefox / files / wallpaper picker |
| Super+d | nwg-displays (way-displays owns the active layout) |
| Alt+Shift+c / Alt+Shift+r | Reload Sway / restart Waybar |
| Alt+1…0 / Alt+Shift+1…0 | Switch workspace / move container |
| Three-finger swipe | Previous/next numbered workspace |
| Alt+f | Toggle fullscreen |
| Alt+p / Super+p / display key | Toggle wl-mirror on the external output |
| Super+l | Lock screen |
| Super+g | Restore a frozen fullscreen game's presentation |
| Super+Space | Cycle US/Arabic/French keyboard layouts |
| Print | Save a screenshot in `~/Pictures/Screenshots` |

Mirroring uses wl-mirror with fit scaling to preserve the laptop's complete
16:10 image on a 16:9 external display. The workspace guard keeps the mirror on
its dedicated workspace and other workspaces on the main display while mirroring.
A manual toggle off remains off through subsequent hotplug callbacks until
re-enabled or a new session starts.

`sway-display-toggle --main OUTPUT --ext OUTPUT` selects the capture source and
destination when several monitors are connected. The workspace guard follows
that pair. Hotplug recovery checks the actual mirror window and removes stale
mirrors when a source or destination becomes unavailable. Mirror logs are in
`$XDG_RUNTIME_DIR/wl-mirror.log`.

The idle helper locks after five minutes, powers outputs off after ten, and
locks before sleep. Fullscreen visible windows inhibit idle; the permanently
fullscreen mirror is explicitly excluded. The Rust gamepad guard reads evdev
activity and does not take ownership of existing inhibitors. Access to gamepad
input devices must be available to the user.

`Super+g` preserves ordinary and global fullscreen. The optional freeze watcher
is disabled because unchanged pixels alone cannot reliably prove a game froze.

## Validate

```sh
cargo test --locked --offline --manifest-path rust/sway-helpers/Cargo.toml
cargo clippy --locked --offline --manifest-path rust/sway-helpers/Cargo.toml --all-targets -- -D warnings
cargo fmt --manifest-path rust/sway-helpers/Cargo.toml -- --check
cargo test --locked --offline --manifest-path rust/waybar-helpers/Cargo.toml
cargo clippy --locked --offline --manifest-path rust/waybar-helpers/Cargo.toml --all-targets -- -D warnings
python3 tests/shell_helpers.py
# Isolated three-display integration test (this machine's Intel render node):
WLR_RENDER_DRM_DEVICE=/dev/dri/renderD128 python3 tests/displays.py
bash tests/install.sh
sway -c ~/.config/sway/config --validate
~/.local/bin/sway-gamepad-idle-guard --status
~/.local/bin/sway-lock --dry-run
```

Omit `--offline` for Cargo's initial dependency download. The installation test
uses cached crates and temporary destinations, leaving the live configuration
alone. The audit includes tests with virtual outputs and synthetic gamepad events;
physical TV/controller and suspend/resume behavior still need hardware testing.

License: MIT
