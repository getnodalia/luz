# Font setup for local packaging; dot-sourced by package.ps1.
function Initialize-NodaliaFonts([string] $WorkspaceRoot) {
  $fontDir = $env:NODALIA_FONTS_DIR
  if (-not $fontDir) { $fontDir = Join-Path $WorkspaceRoot '../nodalia-fonts' }
  if (-not [IO.Path]::IsPathRooted($fontDir)) { $fontDir = Join-Path $WorkspaceRoot $fontDir }

  if (-not $env:NODALIA_FONTS_DIR -and -not (Test-Path -LiteralPath $fontDir)) {
    # Same revision as release.yml and packaging/fonts.sh. Existing checkouts stay untouched.
    $revision = '7a66e2c6c6d60ac6417caef3e339e186d003b5b8'
    Write-Output "==> downloading nodalia-fonts ($revision) to $fontDir"
    & git clone --no-checkout --single-branch --branch nodalia/fonts https://github.com/getnodalia/foundation $fontDir
    if ($LASTEXITCODE -ne 0) { throw 'Could not download nodalia-fonts; provide a complete checkout with NODALIA_FONTS_DIR' }
    & git -C $fontDir checkout --detach $revision
    if ($LASTEXITCODE -ne 0) { throw 'Could not check out nodalia-fonts; provide a complete checkout with NODALIA_FONTS_DIR' }
  }

  $manifest = Join-Path $fontDir 'fonts/manifest.txt'
  if (-not (Test-Path -LiteralPath $manifest -PathType Leaf)) {
    throw "Missing $manifest; set NODALIA_FONTS_DIR to a complete nodalia-fonts checkout"
  }
  $env:NODALIA_FONTS_DIR = (Resolve-Path -LiteralPath $fontDir).Path
  $env:NODALIA_FONTS_REQUIRED = '1'
  Write-Output "==> embedding fonts from $env:NODALIA_FONTS_DIR"
}
