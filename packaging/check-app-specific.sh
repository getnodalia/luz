#!/usr/bin/env bash
# App-specific installer and package assertions retained alongside shared packaging lint.
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."

for want in '<UIRef Id="LuzUI" />' 'Value="InstallDirDlg"' '<Property Id="WIXUI_INSTALLDIR" Value="INSTALLFOLDER" />' \
    'Key="Software\Luz" Name="InstallFolder"' 'Value="LaunchLuz"'; do
  grep -qF "$want" packaging/windows/luz.wxs || { echo "::error::luz.wxs lost: $want" >&2; exit 1; }
done
grep -qF '<DialogRef Id="ExitDialog" />' packaging/windows/luz.wxs \
  || { echo "::error::ExitDialog is not referenced; installs and removes end without a completion dialog" >&2; exit 1; }

# The macOS bundle must declare only file types Luz can open. It used to carry Lienzo's
# `.pcraft` document type and imported PSB/QOI declarations, so Launch Services typed Lienzo
# documents as Luz's and `.pcraft` had three claimant apps (issue #1951). plistlib parses the
# template as a property list too, which is the syntax check package.sh's `plutil -lint` repeats on
# the substituted file.
python3 - <<'PYEOF'
import plistlib
with open("packaging/macos/Info.plist.in", "rb") as source:
    plist = plistlib.load(source)
for key, value in {
    "CFBundleIdentifier": "dev.danielmiguelt.Luz",
    "CFBundleExecutable": "Luz",
    "CFBundleIconFile": "Luz",
    "LSMinimumSystemVersion": "11.0",
}.items():
    assert plist.get(key) == value, f"Info.plist {key}: {plist.get(key)!r}, expected {value!r}"
for key in ("CFBundleShortVersionString", "CFBundleVersion", "LuzVersion", "LuzBuildCommit"):
    assert plist.get(key), f"Info.plist lost {key}"
declared = set()
for section in ("UTExportedTypeDeclarations", "UTImportedTypeDeclarations"):
    for entry in plist.get(section, []):
        tags = entry.get("UTTypeTagSpecification", {})
        declared.update(tags.get("public.filename-extension", []))
        declared.update(tags.get("public.mime-type", []))
        declared.add(entry.get("UTTypeIdentifier", ""))
unsupported = {name.lower() for name in declared} & {
    "pcraft",
    "psb",
    "qoi",
    "application/x-luz",
    "dev.danielmiguelt.Luz.document",
    "dev.danielmiguelt.Luz.qoi",
    "com.adobe.photoshop-large-image",
}
assert not unsupported, f"Info.plist declares types Luz cannot open: {sorted(unsupported)}"
print("Luz macOS bundle types ok")
PYEOF
