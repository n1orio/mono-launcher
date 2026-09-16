#!/bin/bash
set -euo pipefail

# Build the Flatpak
flatpak-builder --force-clean --user flatpak-build flatpak/manifest.json

# Export the Flatpak
flatpak build-export flatpak-build flatpak/manifest.json --force-clean

# Install the Flatpak locally
flatpak --user install --user flatpak-build/ru.mono.launcher.flatpak
