# shellcheck shell=bash
# Font setup for local packaging; sourced after packaging/env.sh.
prepare_fonts() {
  local font_dir="${NODALIA_FONTS_DIR:-$ROOT/../nodalia-fonts}"
  # Relative overrides are documented relative to the workspace, regardless of the caller's cwd.
  case "$font_dir" in
    /*) ;;
    *) font_dir="$ROOT/$font_dir" ;;
  esac

  if [[ -z "${NODALIA_FONTS_DIR:-}" ]] && [[ ! -e "$font_dir" ]]; then
    # Same revision as release.yml. Never update or reset an existing checkout.
    local revision=7a66e2c6c6d60ac6417caef3e339e186d003b5b8
    echo "==> downloading nodalia-fonts ($revision) to $font_dir"
    if ! git clone --no-checkout --single-branch --branch nodalia/fonts https://github.com/getnodalia/foundation "$font_dir" ||
       ! git -C "$font_dir" checkout --detach "$revision"; then
      echo "error: could not prepare nodalia-fonts; provide a complete checkout with NODALIA_FONTS_DIR" >&2
      return 1
    fi
  fi

  if [[ ! -f "$font_dir/fonts/manifest.txt" ]]; then
    echo "error: missing $font_dir/fonts/manifest.txt; set NODALIA_FONTS_DIR to a complete nodalia-fonts checkout" >&2
    return 1
  fi
  NODALIA_FONTS_DIR="$(cd "$font_dir" && pwd)" || return 1
  export NODALIA_FONTS_DIR
  # Fail the build if required faces are missing, rather than ship empty glyph boxes.
  export NODALIA_FONTS_REQUIRED=1
  echo "==> embedding fonts from $NODALIA_FONTS_DIR"
}
