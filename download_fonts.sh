#!/bin/bash

mkdir -p fonts

OPENSANS_FONT_URL="https://github.com/google/fonts/raw/refs/heads/main/ofl/opensans/OpenSans%5Bwdth,wght%5D.ttf"
OPENSANS_ITALIC_FONT_URL="https://github.com/google/fonts/raw/refs/heads/main/ofl/opensans/OpenSans-Italic%5Bwdth,wght%5D.ttf"
./download_font.sh "./fonts" "OpenSansVariable.ttf" "$OPENSANS_FONT_URL"
./download_font.sh "./fonts" "OpenSansItalicVariable.ttf" "$OPENSANS_ITALIC_FONT_URL"
