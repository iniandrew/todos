#!/usr/bin/env bash
# Prints GitHub release notes for a version: its CHANGELOG.md section plus
# download/installation notes.
#
# Usage: scripts/release-notes.sh <version>      (e.g. 0.2.0)
set -euo pipefail

version="${1:?usage: $0 <version>}"
root="$(cd "$(dirname "$0")/.." && pwd)"

# Lines after "## [<version>]" up to the next "## [" heading.
section="$(awk -v v="$version" '
  index($0, "## [" v "]") == 1 { found = 1; next }
  found && /^## \[/ { exit }
  found { print }
' "$root/CHANGELOG.md")"

if [ -z "${section//[[:space:]]/}" ]; then
  echo "error: no CHANGELOG.md entry for version $version" >&2
  exit 1
fi

printf '%s\n' "$section" | sed -e '/./,$!d'   # drop leading blank lines
cat <<NOTES

## Download

| Platform | File |
| --- | --- |
| Windows 10/11 | \`Todos_${version}_x64-setup.exe\` or \`Todos_${version}_x64_en-US.msi\` |
| macOS (Apple Silicon & Intel) | \`Todos_${version}_universal.dmg\` |
| Debian / Ubuntu | \`Todos_${version}_amd64.deb\` |
| Fedora / openSUSE | \`Todos-${version}-1.x86_64.rpm\` |
| Any Linux distro | \`Todos_${version}_amd64.AppImage\` |
| Arch Linux | \`todos-bin\` PKGBUILD in \`packaging/arch/todos-bin\`, or the portable \`todos-${version}-linux-x86_64.tar.gz\` |

> [!NOTE]
> The installers are not code-signed yet.
> - **macOS:** right-click the app and choose **Open**, or run \`xattr -cr /Applications/Todos.app\`.
> - **Windows:** if SmartScreen appears, choose **More info → Run anyway**.

**Full changelog:** [CHANGELOG.md](https://github.com/iniandrew/todos/blob/v${version}/CHANGELOG.md)
NOTES
