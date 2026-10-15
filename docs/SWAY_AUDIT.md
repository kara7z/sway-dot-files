# Sway review and Rust migration

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
