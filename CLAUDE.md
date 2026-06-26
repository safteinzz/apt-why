# whypkg

## Overview
`whypkg` ("why the hell is this package here?") is a fast, cross-distro CLI that investigates
installed packages: did *you* install it or did something pull it in, when, alongside what, what
needs it, and what it needs. It's a Rust rewrite of the original bash `apt-why`/`apt-pending`
scripts (kept in `legacy/`). Two modes share one engine: an interactive `ratatui` browser
(default) and a `pending` report classifying upgradable packages by why they're present.

## Tech stack
Rust 2021. clap (derive) · ratatui + crossterm (TUI) · nucleo-matcher (fuzzy, no external fzf) ·
colored · chrono · flate2. Crate + binary both `whypkg`. License AGPL-3.0-only.

## Layout
- `src/main.rs` — clap CLI. No subcommand = browse (`--upgradable` flag); `pending`; `update`.
- `src/model.rs` — `Package` and `World` (in-memory packages + dep graph, built once at startup).
- `src/engine.rs` — distro-agnostic logic: `bfs_root` (the "why is this here" origin trace),
  foundation detection, `same_session`, `is_kernel_pkg`, `format_size`, `relative_time`.
- `src/backend/mod.rs` — `Backend` trait, `detect()`, `capture()` (forces `LC_ALL=C`).
- `src/backend/{apt,pacman,dnf}.rs` — one file per package manager; parsing is pure functions.
- `src/commands/browse.rs` — the TUI (fuzzy list + dossier + breadcrumb nav).
- `src/commands/pending.rs` — the upgrade report. `mod.rs` has `load_world()`.
- `tests/fixtures/` — real `pacman`/`dnf` output captured from containers, used by unit tests.
- `legacy/` — original bash scripts; excluded from the published crate.

## Commands
- Build: `cargo build` / `cargo build --release`
- Test: `cargo test` (13 tests; backend parsers tested against `tests/fixtures/` via `include_str!`)
- Lint: `cargo clippy`
- Run: `./target/release/whypkg`, `whypkg --upgradable`, `whypkg pending [--quick|--kernel|--apps|--auto|--sizes]`

## Conventions
- Commits: short, single-line, conventional prefix (`feat:`/`fix:`/`chore:`…). NEVER add a
  `Co-Authored-By` trailer or a verbose body.
- Release: bump `Cargo.toml` version → commit → `cargo publish` → `git tag vX.Y.Z` && push tag.
- Mirror the `sluuz` crate's conventions (clap-derive subcommand per file with `Args`+`run()`,
  heavy *why*-focused doc comments).
- Adding a distro = one new `src/backend/<name>.rs` + one line in `detect()`. Nothing else changes.

## Gotchas
- whypkg NEVER syncs or modifies the system. Upgradables reflect the user's last sync
  (`pacman -Qu`, `apt list --upgradable`, `dnf repoquery --upgrades --cacheonly`) — like
  `apt list --upgradable` needs `apt update` first.
- `capture()` forces `LC_ALL=C` so field labels, dates, and sizes parse regardless of locale.
- dnf/rpm deps are *capabilities*, not package names — resolved to packages via a PROVIDES+FILENAMES
  provider map (`src/backend/dnf.rs`). pacman gives `Required By` + `Install Reason` natively.
- The browser needs a real TTY (errors cleanly otherwise). `Ctrl+J` collides with Enter unless the
  terminal speaks the kitty keyboard protocol; arrows / `Ctrl+P`/`Ctrl+N` always work.
- Test other distros with podman: build whypkg inside `archlinux`/`fedora`/`cachyos/cachyos`
  containers (`CARGO_TARGET_DIR=/tmp/...`); regenerate fixtures from there.
- Local dir is `~/projects/apt-why` but the crate/repo/remote are all `whypkg`.
