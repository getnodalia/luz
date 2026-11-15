# Luz app icon

A **lynx**, frontal head-and-shoulders portrait with an alert gaze, in the style of an engraving: the
Crafting Apps "owl template" (full-bleed colour field, tight portrait, the figure in paper with ink line
work and a crisp ink contour, no roundel or border).

## Palette (exactly three colours)

| Colour | Hex | Used for |
|---|---|---|
| Ink | `#0b0b0c` | line work and the contour |
| Paper | `#efe9dc` | the figure (the lynx) |
| Luz amber | `#f2a516` | the full-bleed field (the app colour) |

## Geometry

`viewBox="0 0 512 512"`, a rounded-square tile with `rx=112`, clipped; the colour fills the whole tile.
macOS files (`.icns`, `luz-macos-512.png`) add Apple's transparent margin (an 824/1024 body);
Windows and Linux files use the full-bleed tile so the lynx reads at 16-48 px.

## Provenance

The owner's original drawing, made in Nodalia (2880 px, keyed to the three colours) and
vectorised. `luz.svg` here is the canonical artwork. Licence:
`LICENSE.txt` (MIT OR Apache-2.0, like the repo).

## Files

| File | What |
|---|---|
| `luz.svg` | canonical vector, traced at 2048 px (about 770 KB) |
| `luz-small.svg` | lighter vector, traced at 1024 px (about 360 KB) |
| `luz-1024.png` | 1024 px render (store listings, docs) |
| `luz-macos-512.png` | runtime window/Dock icon on macOS (embedded by `apps/luz/src/main.rs`) |
| `luz.icns` | macOS bundle icon (`CFBundleIconFile`) |
| `luz.ico` | Windows icon, 16-256 px, embedded in `luz.exe` by `apps/luz/build.rs` |
| `hicolor/<n>x<n>/apps/dev.danielmiguelt.Luz.png` | Linux icon theme, 16-512 px; the 256 px one is also the runtime window icon on Windows and Linux |
| `hicolor/scalable/apps/dev.danielmiguelt.Luz.svg` | Linux scalable icon (the small SVG) |

The app id is `dev.danielmiguelt.Luz`: the Wayland app id, the `.desktop` file
(`packaging/linux/dev.danielmiguelt.Luz.desktop`, `Icon=dev.danielmiguelt.Luz`) and the hicolor icon
name. To install on Linux, copy `hicolor/` into `/usr/share/icons/hicolor/` (or `~/.local/share/icons/hicolor/`)
and the `.desktop` file into `applications/`.

## Regenerate

Edit `luz.svg` (or `luz-small.svg`), then run `packaging/icons.sh` (needs `resvg`; `iconutil`
on macOS for the `.icns`; the `.ico` is packed by `cargo xtask ico`). Every output is committed.
