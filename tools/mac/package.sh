#!/usr/bin/env bash
# Builds TrioKey.app (universal: Apple silicon and Intel) and TrioKey-<version>-macos.dmg on a macOS machine
# (docs/ARCHITECTURE.md, 12. macOS app). Takes no secrets: the app is signed ad hoc and not notarized, so people open it
# with "Open Anyway". Run from anywhere: bash tools/mac/package.sh
set -euo pipefail

cd "$(dirname "$0")/../.."
version=$(sed -n 's/^version = "\(.*\)"$/\1/p' crates/triokey-mac/Cargo.toml | head -n 1)
[ -n "$version" ] || { echo "no version in crates/triokey-mac/Cargo.toml" >&2; exit 1; }

# Rust's floor is 10.12 on Intel and 11.0 on Apple silicon; one floor for both slices
export MACOSX_DEPLOYMENT_TARGET=11.0
for t in aarch64-apple-darwin x86_64-apple-darwin; do
  cargo build -p triokey-mac --release --locked --target "$t"
done

out=target/mac
app="$out/TrioKey.app"
rm -rf "$out"
mkdir -p "$app/Contents/MacOS" "$app/Contents/Resources"

lipo -create -output "$app/Contents/MacOS/TrioKey" \
  target/aarch64-apple-darwin/release/triokey-mac target/x86_64-apple-darwin/release/triokey-mac

# the bundle identifier is fixed: the Accessibility grant and the single-instance check depend on it
cat > "$app/Contents/Info.plist" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleIdentifier</key>
  <string>io.github.ved0el.triokey</string>
  <key>CFBundleName</key>
  <string>TrioKey</string>
  <key>CFBundleExecutable</key>
  <string>TrioKey</string>
  <key>CFBundleShortVersionString</key>
  <string>$version</string>
  <key>CFBundleVersion</key>
  <string>$version</string>
  <key>CFBundlePackageType</key>
  <string>APPL</string>
  <key>CFBundleIconFile</key>
  <string>TrioKey</string>
  <key>LSUIElement</key>
  <true/>
  <key>LSMinimumSystemVersion</key>
  <string>11.0</string>
  <key>NSHumanReadableCopyright</key>
  <string>© 2026 ved0el</string>
</dict>
</plist>
EOF

# icon: every size iconutil knows up to 512 px, scaled down from the 512 px brand icon (there is no 1024 px source)
iconset="$out/TrioKey.iconset"
mkdir -p "$iconset"
for s in 16 32 128 256 512; do
  sips -z "$s" "$s" assets/brand/icon-512.png --out "$iconset/icon_${s}x${s}.png" > /dev/null
  if [ $((s * 2)) -le 512 ]; then
    sips -z $((s * 2)) $((s * 2)) assets/brand/icon-512.png --out "$iconset/icon_${s}x${s}@2x.png" > /dev/null
  fi
done
iconutil -c icns "$iconset" -o "$app/Contents/Resources/TrioKey.icns"
rm -rf "$iconset"

# last, once everything is in place: arm64 code must be signed to run, and the linker's ad-hoc signature covers only the
# executable, not the bundle around it
codesign --force --deep -s - "$app"

stage="$out/dmg"
mkdir -p "$stage"
cp -R "$app" "$stage/"
ln -s /Applications "$stage/Applications"
cat > "$stage/DOC TRUOC - READ FIRST.txt" <<'EOF'
TrioKey cho macOS
=================

1. Mở ứng dụng chưa được Apple công chứng (notarize)
   TrioKey không có chứng chỉ Apple Developer nên macOS chặn lần mở đầu tiên.
   - Mở TrioKey một lần (macOS báo không mở được, bấm Xong).
   - Vào Cài đặt hệ thống → Quyền riêng tư & Bảo mật, kéo xuống, bấm "Vẫn mở" (Open Anyway).
   - Bấm "Mở" (Open) ở hộp thoại tiếp theo.

2. Luôn kéo TrioKey vào thư mục Applications trước, rồi mở từ đó.
   Đừng chạy thẳng từ file dmg hay thư mục Downloads.

3. Cấp quyền Trợ năng (Accessibility)
   TrioKey phải thấy phím bạn gõ để bỏ dấu; nó không gửi chữ bạn gõ đi đâu cả.
   Cài đặt hệ thống → Quyền riêng tư & Bảo mật → Trợ năng → bật TrioKey.
   Nếu không thấy TrioKey trong danh sách, bấm dấu + và chọn TrioKey trong Applications.
   Menu của TrioKey (chữ V hoặc E trên thanh menu) có mục "Hướng dẫn cấp quyền…".

4. Cài bản mới
   - Thoát TrioKey cũ (menu → Thoát).
   - Chép TrioKey mới đè lên bản trong Applications.
   - Trong Trợ năng, xoá TrioKey (nút −) rồi thêm lại (nút +). Bản mới có chữ ký khác nên quyền cũ không còn dùng được.
   KHÔNG BAO GIỜ tắt hoặc xoá TrioKey khỏi danh sách Trợ năng khi TrioKey đang chạy: việc đó có thể làm bàn phím và
   chuột đứng cho tới khi khởi động lại máy. Luôn thoát TrioKey trước.


TrioKey for macOS
=================

1. Opening an app that is not notarized
   TrioKey has no Apple Developer certificate, so macOS blocks the first launch.
   - Open TrioKey once (macOS says it cannot be opened; click Done).
   - Go to System Settings → Privacy & Security, scroll down, click "Open Anyway".
   - Click "Open" in the next prompt.

2. Always move TrioKey into the Applications folder first and run it from there.
   Do not run it straight from the dmg or from Downloads.

3. Granting Accessibility
   TrioKey has to see the keys you type to place the tone marks; it sends none of your typing anywhere.
   System Settings → Privacy & Security → Accessibility → turn TrioKey on.
   If TrioKey is not in the list, click + and choose TrioKey in Applications.
   TrioKey's menu (the V or E in the menu bar) has "Hướng dẫn cấp quyền…" (permission guide).

4. Installing a new version
   - Quit the old TrioKey (menu → Thoát).
   - Copy the new TrioKey over the one in Applications.
   - In Accessibility, remove TrioKey (− button) and add it again (+ button). A new build has a different signature,
     so the old grant no longer applies.
   NEVER turn off or remove TrioKey in the Accessibility list while TrioKey is running: it can freeze the keyboard and
   mouse until the Mac restarts. Always quit TrioKey first.
EOF

dmg="$out/TrioKey-$version-macos.dmg"
# hosted macOS runners now and then fail this with "Resource busy"
for i in 1 2 3; do hdiutil create -volname "TrioKey $version" -srcfolder "$stage" -format UDZO -ov "$dmg" && break; sleep 5; done
[ -f "$dmg" ] || { echo "hdiutil create failed" >&2; exit 1; }
rm -rf "$stage"

echo "app: $app"
echo "dmg: $dmg"
