# Waybar Python-to-Rust migration — 2026-10-15

Installed the native executable at `~/.local/libexec/waybar-helpers`, with
waybar-dropdown, waybar-media, waybar-wifi-password, and waybar-helpers symlinks
in `~/.local/bin`. Source, Cargo.lock, tests and installer are included in
`~/sway-dot-files/rust/waybar-helpers`.

Converted the Python GTK dropdown, playerctl media JSON loop, legacy Wi-Fi
password dialog, and both embedded Python keyboard-layout parsers. The actual
Waybar config remains JSONC and styling remains CSS. Shell menu/action launchers
are retained. No Python runtime helper remains in the active Waybar configuration.

Preserved the live top bar, current styling, fast Wi-Fi opening, periodic refresh,
selected-network persistence, empty/error scan handling, password Connect button,
and stay-open/refocus behavior. Fixed monitor-relative margins and bounded
keyboard-menu width to the dropdown's supported 560-pixel maximum. A closed
menu cancels and waits for its worker to terminate the scan process group.

The main dotfiles installer builds both Rust projects first, supports fresh
crate downloads and optional offline builds, deploys native command links in both
copy and symlink modes, and backs up/retire old Python runtime entry points.
Optional Python GUI diagnostics live under repository tests, not installed helpers.

Verification:
- 6 Rust unit tests and 3 CLI integration tests passed.
- Clippy with warnings as errors and rustfmt passed.
- 94 shell checks passed using fake menus, NetworkManager and Sway inputs.
- Isolated native Wayland menu screenshot confirmed top positioning at
  (1488, 5)–(1728, 187) on a 1920×1200 virtual output.
- Native GUI checks passed for periodic refresh, clearing results, resizing,
  masked password cancellation, SIGTERM handling, and scan-process cleanup.
- Private XWayland GUI driver: 11 passed, 0 failed, 2 skipped (X11 has no
  layer-shell geometry; its first synthetic Escape was not delivered).
  Escape, Cancel, Refresh, password submission/debug redaction, focus persistence/default dismissal, and arrow-key
  selection passed in subsequent phases.
- Copy/symlink installations, upgrades, old symlink targets, Python retirement,
  timestamped backups, paths with spaces, and Rust entry points were verified.

The migration did not connect/disconnect networks or perform power actions.
Backups of the prior live config and dropdown are in config/ and bin/ here.

Waybar reloaded successfully with SIGUSR2; one Waybar process remained running.
The installed executable matches the final release build, including redacted
password keystroke logging. The config and keyboard menu match the dotfiles.
