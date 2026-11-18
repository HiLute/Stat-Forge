#!/usr/bin/env bash
# Собирает «Stat Forge.app» из готового бинарника. Запускается на macOS
# (нужны sips и iconutil).
#
#   packaging/macos/make-app.sh <бинарник> <версия> <каталог-назначения>
set -euo pipefail

binary=$1
version=$2
out_dir=$3

root=$(cd "$(dirname "$0")/../.." && pwd)
app="$out_dir/Stat Forge.app"

rm -rf "$app"
mkdir -p "$app/Contents/MacOS" "$app/Contents/Resources"
cp "$binary" "$app/Contents/MacOS/stat-forge"
chmod +x "$app/Contents/MacOS/stat-forge"

# Иконка: набор размеров из PNG 1024×1024 → AppIcon.icns.
iconset=$(mktemp -d)/AppIcon.iconset
mkdir -p "$iconset"
for size in 16 32 128 256 512; do
    sips -z "$size" "$size" "$root/assets/icon.png" --out "$iconset/icon_${size}x${size}.png" >/dev/null
    double=$((size * 2))
    sips -z "$double" "$double" "$root/assets/icon.png" --out "$iconset/icon_${size}x${size}@2x.png" >/dev/null
done
iconutil -c icns "$iconset" -o "$app/Contents/Resources/AppIcon.icns"

cat > "$app/Contents/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleName</key>
    <string>Stat Forge</string>
    <key>CFBundleDisplayName</key>
    <string>Stat Forge</string>
    <key>CFBundleIdentifier</key>
    <string>io.github.hilute.stat-forge</string>
    <key>CFBundleExecutable</key>
    <string>stat-forge</string>
    <key>CFBundleIconFile</key>
    <string>AppIcon</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleShortVersionString</key>
    <string>$version</string>
    <key>CFBundleVersion</key>
    <string>$version</string>
    <key>LSMinimumSystemVersion</key>
    <string>11.0</string>
    <key>NSHighResolutionCapable</key>
    <true/>
</dict>
</plist>
PLIST

# Ad-hoc подпись: без неё приложение не запустится на Apple Silicon.
# Это не подпись разработчика Apple — Gatekeeper всё равно предупредит
# при первом запуске.
codesign --force --deep --sign - "$app"

echo "$app"
