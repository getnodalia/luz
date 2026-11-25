# Nodalia migration: Luz

Category: secondary app.
Status: Base Nodalia preparada, as of 2026-11-13. Evidence: the pull request that adds this file.

## Current state

Pure-Rust photo library and raw developer (egui), MIT OR Apache-2.0. Cargo git dependencies come from getnodalia/foundation and nodalia-design. The gate, FreeBSD,
Nix, packaging lint, AGENTS.md and git-sources checks call getnodalia/foundation's reusable
workflows pinned by commit. SonarCloud was removed; CodeQL and Scorecard remain.

## Desired state

No workflow, dependency or document depends on storytold; Luz has its own release workflow; the
interface follows Foundation's [NDL standard](https://github.com/getnodalia/foundation/blob/main/standards/ndl.md).

## Inherited dependencies still pending

| Dependency | Where | Replacement | Blocked on |
|---|---|---|---|
| `storytold/craft-common-actions` | `.github/workflows/windows-arm64.yml` | Foundation reusable workflows (release, refresh-contributors, windows-arm64) | writing and testing them |
| `storytold/craft-fonts` | `crates/engine/build.rs`, `flake.nix`, `nix/package.nix`, `packaging/env.sh`, `packaging/windows/package.ps1` | getnodalia/foundation `vendor/craft-fonts/main` | repointing |
| `storytold/craft-libs` | `nix/package.nix`, `packaging/linux/flatpak/README.md`, `packaging/linux/flatpak/cargo-sources.json` | to decide | — |
| `storytold/egui` | `packaging/linux/flatpak/cargo-sources.json` | to decide | — |
| `storytold/egui.git` | `packaging/linux/flatpak/cargo-sources.json` | to decide | — |
| `getnodalia/luz` | `.github/workflows/upstream-sync.yml`, `Cargo.toml`, `packaging/linux/dev.danielmiguelt.Luz.metainfo.xml.in`, `packaging/linux/flatpak-oci.sh`, `packaging/linux/package.sh`, `packaging/windows/sign.ps1` | none: intended upstream sync | — |
| `storytold/vectorcraft` | `packaging/macos/dmg/README.md` | to decide | — |
| `storytold/winit` | `packaging/linux/flatpak/cargo-sources.json` | to decide | — |
| `storytold/winit.git` | `packaging/linux/flatpak/cargo-sources.json` | to decide | — |

## Technical debt

- None left from the rename: every name is Luz's (`scripts/rebrand.py check`). File signatures
  keep upstream's values on purpose (`rebrand: keep`), so older libraries and presets open.

## Reusable Nodalia components

getnodalia/foundation (vendored crates, fonts, translations, reusable workflows);
getnodalia/nodalia-design (theme, icons).

## NDL

Done (2026-11-18): Nodalia's palette for the chrome (`nodalia-theme`, `Accent::Luz`, creator
suite) with the photo surfaces kept neutral; panels as rounded islands on the window colour; a
hamburger menu with Preferences, Keyboard shortcuts and About; the theme chosen only in
Preferences; NDL window controls on Linux only, with a preference for the desktop's own; the
menus exported over D-Bus to the desktop's top bar on Linux (`apps/luz/src/linux_menu.rs`).

Done (2026-11-25): the flat general-purpose icons come from the NDL set
(`crates/ui-egui/src/icons.rs`, `ndl_name`); Luz keeps its own drawings only for photo-specific
ones (curves, masks, grading wheels, flags, filled stars).

Still to do: the creator suite's shared Home sidebar and dialogs.

## Risks and priorities

1. Releases still run the inherited release workflow.
2. Windows ARM64 packaging still runs the inherited workflow.

## Recommended order

1. A Foundation release workflow, tested with a dry run, then Luz's release on it.
2. A Foundation Windows ARM64 workflow.
3. The creator suite's shared Home and dialogs.

## What the first agent should do

1. Read AGENTS.md and this file.
2. Pick the first item of "Recommended order".

## Releases

Luz releases through `release-auto.yml` like every Nodalia app (Foundation's `release-tag` and
`release-publish`, with `release-linux.yml`, `release-macos.yml` and `release-windows.yml`). The
packages are named `luz-X.Y.Z-…`.
