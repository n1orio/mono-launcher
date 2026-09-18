#!/bin/bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
cd "$PROJECT_DIR"

flatpak-builder --force-clean --disable-rofiles-fuse --user --install-deps-from=flathub --assumeyes \
  --jobs="${FLATPAK_JOBS:-4}" --repo=.flatpak-builder/repo \
  flatpak-build flatpak/manifest.json
flatpak build-bundle .flatpak-builder/repo flatpak-build/ru.mono.launcher.flatpak \
  ru.mono.launcher stable --runtime-repo=https://flathub.org/repo/flathub.flatpakrepo
