# Changelog

All notable changes to Todos are documented here.
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project follows [Semantic Versioning](https://semver.org/).

## [0.2.0] - 2026-11-05

### Added

- **In-app updates**: Todos checks for new versions (at startup and once a day), shows what's new, and on
  Windows, macOS and the AppImage installs the update and restarts with one click. Updates are signature-checked
  before installing. Copies installed with `.deb`, `.rpm` or the Arch package get a notification with a download
  link instead. Turn automatic checks off in *Settings → Updates*.
- **Backup & restore** in *Settings → Data*:
  - Export everything (lists, tasks, steps, settings) to a JSON backup file.
  - Import a backup with a preview of its contents, then choose **Merge** (adds and updates, never deletes)
    or **Replace all**. A safety copy of your current data is saved before replacing.
- **Export to CSV** to open your tasks in Excel, Numbers or Google Sheets.
- **Daily automatic backups**, keeping the last 7 days. Restore one from *Settings → Data*.
- **Open data folder** button, to find your database and backups.
- Portable Linux tarball (`todos-<version>-linux-x86_64.tar.gz`) attached to every release.
- `todos-bin` PKGBUILD for Arch Linux, which installs the prebuilt binary without compiling.

## [0.1.0] - 2026-11-05

First release.

- Smart lists: My Day (resets every day), Important, Planned (grouped by date) and Tasks, plus custom lists
  with 12 colors and 29 icons.
- Task details: steps, due dates, reminders with native desktop notifications, notes, and importance.
- Apple-inspired interface with light, dark and system appearance.
- Drag and drop to reorder tasks and lists, right-click menus, search, undo after delete, and keyboard
  shortcuts.
- English and Bahasa Indonesia.
- Installers for Windows (`.msi`, `.exe`), macOS (universal `.dmg`) and Linux (`.deb`, `.rpm`, `.AppImage`).

[0.2.0]: https://github.com/iniandrew/todos/releases/tag/v0.2.0
[0.1.0]: https://github.com/iniandrew/todos/releases/tag/v0.1.0
