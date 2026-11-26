---
title: Install Luz
description: Download Luz for Windows 11, macOS or Linux, install it, and open your photo library.
order: 10
verified: 0.7.1
---

Download the file for your system from the [latest release](https://github.com/getnodalia/luz/releases/latest) and install it. In the file names, `<version>` stands for the version number, for example `0.7.1`. The [Luz page on getnodalia.com](https://getnodalia.com/luz) also links the download.

## Windows 11

1. Open the [latest release](https://github.com/getnodalia/luz/releases/latest).
2. Download `luz-<version>-windows-x64.msi`. It fits most PCs (Intel and AMD, 64-bit).
3. Run the file. The first page asks who to install Luz for. **Only for me** is the default: it needs no administrator permission and installs to `%LOCALAPPDATA%\Programs\Nodalia\Luz`. **Everyone on this computer** asks for an administrator and installs to `C:\Program Files\Nodalia\Luz` (`Program Files (x86)` for the 32-bit build). Upgrades stay in the folder you chose.
4. At the end, choose the option to start Luz.

For a Snapdragon or other Arm PC, download `luz-<version>-windows-arm64.msi` instead. For a 32-bit Windows, download `luz-<version>-windows-x86.msi`.

Upgrading from an installer older than 0.6.0 also keeps Luz in the folder it was installed in. An upgrade uses the folder that the earlier install recorded and does not check that it still exists, so if you deleted that folder, the upgrade creates it again.

To run it without installing, download `luz-<version>-windows-x64-portable.zip`. Portable builds for `arm64` and `x86` are on the release page too.

## macOS

1. Download `luz-<version>-macos-universal.dmg`. It works on Apple silicon and Intel Macs, and is signed and notarized.
2. Open the file and install Luz.

## Linux

The release has these files for `x86_64` and `aarch64` (Arm). Replace `x86_64` with `aarch64` on Arm.

| File                                         | Notes                                                    |
|----------------------------------------------|----------------------------------------------------------|
| `luz-<version>-linux-x86_64.AppImage` | Runs anywhere.                                           |
| `luz-<version>-linux-x86_64.deb`      | For Debian and Ubuntu.                                   |
| `luz-<version>-linux-x86_64.rpm`      | For Fedora, RHEL and openSUSE.                           |
| `luz-<version>-linux-x86_64.tar.gz`   | Unpack anywhere.                                         |

RISC-V and FreeBSD builds are on the release page too. On a Wayland desktop, Luz runs through XWayland when it is available, because only X11 gives it file drag-and-drop.

## Install with Nodalia Launcher

[Nodalia Launcher](https://getnodalia.com/docs/craft-launcher) lists Luz and can install and update it for you. On Windows it installs the release `.msi`.

## In a web browser

The release also has `luz-web-<version>.zip`, a static site that runs Luz in a modern browser. Host it on any static web server.

## First run

1. Start Luz. It opens your library at `~/Pictures/Luz Library`. A new library starts with demo photos.
2. To bring in your own photos, see [Import photos](../guides/import-photos.md).

## Still stuck?

Ask in [Discord](https://github.com/getnodalia/luz/issues) or open an issue in the [issue tracker](https://github.com/getnodalia/luz/issues).
