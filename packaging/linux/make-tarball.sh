#!/usr/bin/env bash
# Packages the release binary into a portable Linux tarball:
#   todos-<version>-linux-x86_64.tar.gz (+ .sha256)
#
# Usage: packaging/linux/make-tarball.sh [output-dir]
# Run from the repository root after `pnpm tauri build`.
set -euo pipefail

root="$(cd "$(dirname "$0")/../.." && pwd)"
out="${1:-$root/dist-linux}"
version="$(jq -r .version "$root/src-tauri/tauri.conf.json")"
arch="$(uname -m)"
name="todos-${version}-linux-${arch}"
bin="$root/src-tauri/target/release/todos"
stage="$out/$name"

[ -x "$bin" ] || { echo "missing $bin — run 'pnpm tauri build' first" >&2; exit 1; }

rm -rf "$stage"
mkdir -p "$stage/icons"
install -m755 "$bin" "$stage/todos"
install -m644 "$root/packaging/linux/todos.desktop" "$stage/todos.desktop"
install -m644 "$root/src-tauri/icons/32x32.png" "$stage/icons/32x32.png"
install -m644 "$root/src-tauri/icons/128x128.png" "$stage/icons/128x128.png"
install -m644 "$root/src-tauri/icons/128x128@2x.png" "$stage/icons/256x256.png"
install -m644 "$root/src-tauri/icons/icon.svg" "$stage/icons/todos.svg"
cat > "$stage/README.txt" <<TXT
Todos ${version} for Linux (${arch})

Requirements: webkit2gtk-4.1, gtk3, libnotify
  Arch:          sudo pacman -S --needed webkit2gtk-4.1 gtk3 libnotify
  Debian/Ubuntu: sudo apt install libwebkit2gtk-4.1-0 libgtk-3-0 libnotify4
  Fedora:        sudo dnf install webkit2gtk4.1 gtk3 libnotify

Run:  ./todos

Blank window (NVIDIA/Wayland)?  WEBKIT_DISABLE_DMABUF_RENDERER=1 ./todos

https://github.com/iniandrew/todos
TXT

(cd "$out" && tar --owner=0 --group=0 -czf "$name.tar.gz" "$name" && sha256sum "$name.tar.gz" > "$name.tar.gz.sha256")
echo "$out/$name.tar.gz"
