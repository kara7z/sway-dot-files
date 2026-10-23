# Sway review and Rust migration

## External display follow-up — 2026-10-23

- Hotplug recovery now checks Sway's actual mirror windows, selected output pair,
  and fullscreen destination. It removes stale windows on disconnect/lid close
  and recreates mirrors which have migrated onto the wrong display.
- Mirror termination is scoped to the current compositor's windows; stopping
  the workspace guard uses its runtime PID/lock instead of a global `pkill`.
- `--main`/`--ext` reach the initial workspace setup. The mirror title records
  both outputs so workspace shortcuts and the daemon preserve the selection,
  even with three displays or after the mirror window migrates on unplug.
- Automatic target selection can now find a connected disabled external.
  Invalid/missing output arguments and identical source/destination are rejected.
- Hotplug/toggle requests wait for the startup lock instead of silently dropping
  events. No-external callbacks return promptly instead of holding it for ten
  seconds. Wallpaper callbacks run independently of mirror recovery.
- Fallback/opt-in 1080p positioning uses logical output width and accepts negative
  coordinates, avoiding gaps/incorrect placement on scaled layouts.
- Mirror logs now use the session runtime directory.

`tests/displays.py` exercised actual wl-mirror and the Rust workspace daemon in
an isolated SwayFX instance with three virtual outputs: disabled-output enabling,
custom source/destination, workspace switching, idempotent ensure, misplaced
mirror recovery, disconnect/reconnect, source disabling, and manual-off behavior.
This machine requires `WLR_RENDER_DRM_DEVICE=/dev/dri/renderD128` for that test:
automatic headless GPU selection chooses NVIDIA while Mesa EGL is selected.

The physical external display was disconnected. Samsung's configured advertised
1080p60 mode and fit scaling are retained; physical cable/TV behavior remains
unverified. Backups: `~/config-backup/sway-display-review-20261023-001539/`.

## Follow-up review — 2026-10-23

The live Sway configuration still validates successfully. Fixed four helper bugs
in the repository and installed copies:

- Manual display toggling defaulted to `cover` on 16:9 outputs, cropping the
  laptop desktop despite the configured `fit` default. Cropping is now opt-in
  with `SWAY_MIRROR_FILL16X9=1`, matching the documented default behavior.
- Mirror workspace daemons inherited the startup lock on file descriptor 9,
  potentially blocking every subsequent toggle or hotplug request. Close that
  descriptor when launching background daemons, including the display-manager
  fallback.
- Wi-Fi lists split escaped SSIDs on colons and retained escape backslashes.
  Unescaped names now occupy the last field, and security lookup matches the
  entire name instead of a substring.
- Menu toggling recognized shell process names but missed `wifi_menu.sh` and
  `power_menu.sh`, which Linux uses when the scripts are executed directly.
  Second-click closing and switching between menus now recognize these names.

`python3 tests/shell_helpers.py` checks these cases with isolated command mocks,
including default fit/explicit cover, daemon descriptor inheritance, special
SSID characters, password routing, and all four same/cross-menu click cases.
The original mirror scripts fail the new regression checks; the fixes pass.
All 15 Sway helper and 9 Waybar helper Rust tests also passed. Physical display
mirroring and actual Wi-Fi connection changes were not exercised by these tests.

Installed helper backups: `~/config-backup/sway-review-20261023-000714/`.

## Original review

Completed on 2026-10-15. The installed compositor is SwayFX 0.6 based on
Sway 1.12. The active configuration is `/home/kara/.config/sway/config`.

## Confirmed issues fixed in the deeper review

- The always-fullscreen wl-mirror window inherited the global idle inhibitor,
  preventing desktop idle locking while mirroring. It now has an explicit
  exception, including on reload and in the gamepad helper.
- Samsung's requested 1920x1200 mode was unavailable. The existing display log
  confirmed fallback to 3840x2160 at 30 Hz. The requested mode is now its
  advertised 1920x1080 at 60 Hz. way-displays confirmed loading the new setting.
- The game recovery shortcut lost global fullscreen state (2 became 0).
  Both ordinary and global fullscreen now restore their original state.
- Workspace moves dropped the `number` semantics and could create a separate
  workspace instead of using an existing name such as `3: code`.
- Swiping from an unnumbered workspace could create workspace 0.
- Wallpaper parsing truncated filenames containing spaces or commas.
- Output overlap was incorrectly treated as a hardware clone. The scripts now
  use wl-mirror. Its documented purpose is to provide mirroring in Sway:
  https://github.com/Ferdi265/wl-mirror
- Mirror validation checked fullscreen state but not its destination output.
- Login/hotplug/toggle operations could race and start multiple mirrors.
- A stale PID belonging to an unrelated process could prevent the old workspace
  daemon from starting. Rust daemon ownership uses an exclusive runtime lock.
- A stable mirrored layout caused unnecessary focus commands. The new guard
  issues no commands when the layout is already correct.
- Mirror cleanup inspected `get_workspaces` as though it contained window
  children. It now inspects the actual tree.
- Mirror recovery renamed a workspace that could contain user applications.
  The Rust implementation moves the mirror container instead.

## Rust installation

Five standalone Python helpers now resolve to the compiled executable
`~/.local/libexec/sway-helpers`:

- sway-workspace
- sway-mirror-workspaces
- sway-mirror-workspace-daemon
- sway-gamepad-idle-guard
- sway-game-freeze-daemon

All eight embedded Python snippets in the display and wallpaper shell helpers
were replaced with `sway-helper-json`, another entry point to the Rust binary.
The original names and Sway keybindings remain usable. The optional game-freeze
watcher remains disabled at startup.

Source, pinned dependencies, build instructions, and installer:
`~/.local/src/sway-helpers/README.md`.

The portable repository copy is `rust/sway-helpers/` in `~/sway-dot-files`.
The main dotfiles installer builds this project and installs its executable
and command symlinks in both copy and symlink modes. Builds use Cargo.lock;
`--offline` is optional so fresh machines can download missing crates.

## Dotfiles integration

- Updated the repository's Sway configuration and all changed shell helpers.
- Replaced the five old Python source files with the Rust project and tests.
- Added the previously omitted way-displays configuration, with portable HOME
  references in callbacks, and installed the workspace include in both modes.
- Fixed symlink installation to include scripts and display settings.
- Back up existing symlinks as well as regular files, avoiding writes through
  links into shared binaries or the source checkout.
- Ensure `--check` never installs packages, even when combined with `--all`.
- Added Rust build dependencies and updated installation/behavior documentation.
- Verified copy, symlink, and upgrade installations in temporary destinations,
  including a destination path containing spaces, backup preservation, existing
  outputs preservation, and all compiled command entry points.
- Reran all 15 Rust tests, Clippy, and formatting checks from the repository.

## Verification

- SwayFX configuration validation passed.
- Rust: 12 unit tests and 3 integration tests passed.
- Clippy with warnings treated as errors and rustfmt checks passed.
- Shell syntax checks passed.
- A separate SwayFX instance with two virtual outputs verified password/game
  rules, ordinary/global fullscreen restoration, a real wl-mirror window's idle
  exception, named workspace moves, and Rust daemon event handling.
- FIFO gamepad events verified acquisition/release, replacement of an existing
  daemon, and SIGTERM cleanup without accessing physical gamepad devices.
- Concurrent mocked hotplug requests produced one mirror and did not leak the
  startup lock into its child process.
- Live Sway reload succeeded and its loaded configuration matches disk.
- Both active custom daemons were verified running the Rust executable.
- Wallpaper and lock wallpaper resolution succeeded. swayidle, Waybar, swaync,
  way-displays, and awww each have one running instance.

The external display and physical controller were disconnected during final
validation. Actual TV output, physical gamepad behavior, and suspend/resume need
verification with those devices in use. Unit and virtual-display tests do not
prove every hardware-dependent path.

## Backups

`backup/` contains the files as they were before the deeper review.
`pre-rust/` contains the repaired helpers immediately before Rust replacement.
`changes.diff` records the deeper review before the migration.
The first review's original Sway config is also preserved at
`~/.config/sway/config.bak.20261015_110915`.

When restoring a helper, copy to a temporary sibling and rename over the
installed symlink. Do not copy directly through the symlink: that would
overwrite the shared executable.
