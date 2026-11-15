# Luz upstream

| Field | Value |
|---|---|
| Upstream | `storytold/lightcraft` |
| Last observed | `e9c29e2c5c5dd1f86cc98677b5dc389f6612fcac` |
| Last evaluated | `e9c29e2c5c5dd1f86cc98677b5dc389f6612fcac` |
| Last integrated | `e9c29e2c5c5dd1f86cc98677b5dc389f6612fcac` merged on 2026-11-10 |
| Skipped | none in the baseline import |
| Adapted | desktop binary name `luz` |
| Conflicting | `README.md` (kept ours), `AGENTS.md` (upstream plus our quality contract) |

Every name is Luz's, internal ones included: crates are `luz-*`, the binaries `luz`,
`luz-cli` and `luz-web`, and the app id `dev.danielmiguelt.Luz`. File signatures stay as
LightCraft wrote them (the catalog snapshot's `"format":"lightcraft-catalog"`, and the
`lightcraft.preset`, `lightcraft.curvePresets`, `lightcraft.maskEffects`, `lightcraft.printTemplate`
and `lightcraft.slideshowTemplate` file formats), so libraries and presets saved before the rename
keep opening. <!-- rebrand: keep -->

## How the sync works

Upstream's code arrives renamed: `scripts/rebrand.py` applies `.github/rebrand.toml` (LightCraft
becomes Luz, internal names included) to upstream's tree, and the weekly sync
(`.github/workflows/upstream-sync.yml`) records the result on the `upstream-luz` branch. Each
commit there has the previous mirror commit and the upstream commit it renames as parents, and
names that commit in an `Upstream-Commit:` trailer. The sync merges the newest mirror commit
into a branch and opens a pull request; files listed in `.github/upstream-keep` keep this
repository's version.

The mirror starts at the renamed `e9c29e2c`, merged into `main` (with this repository's tree)
by the pull request that renamed the app. `python3 scripts/rebrand.py check` fails when an old
name comes back; a line that must keep one says `rebrand: keep`.
