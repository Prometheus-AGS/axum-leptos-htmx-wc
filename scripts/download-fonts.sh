#!/usr/bin/env bash
set -euo pipefail

FONT_DIR="static/fonts"
TMP_DIR=".tmp-fonts"
UA="Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/123 Safari/537.36"

mkdir -p "$FONT_DIR" "$TMP_DIR"

fetch_css() {
  local family_css2="$1"
  local out="$2"
  local url="https://fonts.googleapis.com/css2?family=${family_css2}&display=swap"

  curl -fsSL \
    -H "User-Agent: $UA" \
    -H "Accept: text/css,*/*;q=0.1" \
    "$url" > "$out"
}

download_first_woff2() {
  local css_file="$1"
  local out_file="$2"
  local url
  url="$(grep -oE 'https://fonts\.gstatic\.com/s/[^)]+\.woff2' "$css_file" | head -n 1 || true)"

  if [[ -z "$url" ]]; then
    echo "✖ Could not find a .woff2 URL in $css_file"
    exit 1
  fi

  curl -fsSL \
    -H "User-Agent: $UA" \
    -H "Accept: font/woff2,*/*" \
    -o "$out_file" \
    "$url"
}

echo "▶ Downloading Inter (variable woff2)…"
INTER_CSS="$TMP_DIR/inter.css"
fetch_css "Inter:wght@100..900" "$INTER_CSS"
download_first_woff2 "$INTER_CSS" "$FONT_DIR/Inter-Variable.woff2"
echo "✔ Installed $FONT_DIR/Inter-Variable.woff2"

echo "▶ Downloading Source Sans 3 (variable woff2)…"
SS3_CSS="$TMP_DIR/source-sans-3.css"
fetch_css "Source+Sans+3:wght@200..900" "$SS3_CSS"
download_first_woff2 "$SS3_CSS" "$FONT_DIR/SourceSans3-Variable.woff2"
echo "✔ Installed $FONT_DIR/SourceSans3-Variable.woff2"

echo "▶ Cleanup…"
rm -rf "$TMP_DIR"

echo "✅ Done."