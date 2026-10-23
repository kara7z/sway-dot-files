# Sway helpers in Rust

Local replacements for the Python programs and embedded Python snippets in
Kara's Sway configuration. Existing command names and keybindings are preserved.
The binary talks directly to Sway's i3 IPC socket; Python and the Python i3ipc
package are not needed by these helpers.

Commands provided by one compiled binary:

- `sway-workspace`: execute a workspace command and keep a mirrored layout consistent.
- `sway-mirror-workspaces`: consolidate workspaces, with `--force`, `--main`,
  `--ext`, `--mirror-ws`, `--quiet`, and `--dry-run` options.
- `sway-mirror-workspace-daemon`: subscribe to workspace/window/output events.
  Supports `--foreground`, `--once`, `--once --dry-run`, and `--stop`.
  `--stop` stops only the instance registered in the current runtime directory.
- `sway-gamepad-idle-guard`: read Linux gamepad events and manage only inhibitors
  it acquires. Supports `--status`, `--dry-run`, `--version`, and the existing
  `GAMEPAD_IDLE_GUARD_*` environment overrides. The mirror window is excluded.
- `sway-game-freeze-daemon`: optional fullscreen game recovery. It remains
  disabled at startup, as before. A static image is only a heuristic for a
  frozen frame; `--dry-run` logs proposed recovery actions.
- `sway-helper-json`: structured queries used by the existing shell display and
  wallpaper helpers. Handles output modes, mirror validation, real workspace
  contents, and wallpaper filenames without lossy text parsing. `mirror-pids`
  lists mirrors belonging to the queried compositor; `mirror-pair` reads the
  selected source/destination recorded in the mirror window title.

The remaining shell helpers are retained. `nwg-displays`, Waypaper, and other
third-party applications are outside this helper migration.

## Build and install

```sh
cd ~/.local/src/sway-helpers  # or rust/sway-helpers in sway-dot-files
cargo test --locked --offline
cargo clippy --locked --offline --all-targets -- -D warnings
sh install.sh
```

Dependencies are pinned in `Cargo.lock`. Installation copies the release binary
to `~/.local/libexec/sway-helpers` and replaces the command names with symlinks.
The installer builds with `--locked` and can download missing dependencies on a
fresh machine. Use `sh install.sh --offline` when the Cargo cache is populated.
`--prefix DIRECTORY` changes the installation prefix (default `~/.local`).
`--binary-only` leaves the three shell helpers to the dotfiles installer.

The mirror daemon's PID and lock files now live in `$XDG_RUNTIME_DIR`.
Daemon startup is protected by `flock`; SIGTERM performs orderly cleanup.

## Backups and review

Before the Rust migration, the repaired scripts were saved in:
`~/.local/state/sway-audit/20261015_111244/pre-rust/`.
The same audit directory contains the configuration backup from the start of
the deeper review, the change diff, and isolated Sway runtime test results.

To restore a helper, stop its running daemon if applicable, copy the backed-up
file to a temporary sibling in `~/.local/bin`, and rename it over the symlink.
Copying directly onto a symlink would overwrite the shared Rust binary.
