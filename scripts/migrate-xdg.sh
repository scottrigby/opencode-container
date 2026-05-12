#!/usr/bin/env bash
# SPDX-License-Identifier: Apache-2.0
# Migrate opencode-container data from old platform-native paths to XDG paths.
#
# On macOS, this moves data from ~/Library/Application Support/opencode/ to
# ~/.local/share/opencode/ and ~/.config/opencode/.
#
# Usage: ./scripts/migrate-xdg.sh [--dry-run]

set -euo pipefail

DRY_RUN=false
if [[ "${1:-}" == "--dry-run" ]]; then
    DRY_RUN=true
    echo "[DRY RUN] No files will be moved"
fi

migrate_dir() {
    local src="$1"
    local dst="$2"
    local label="$3"

    if [[ ! -d "$src" ]]; then
        return 0
    fi

    if [[ -d "$dst" ]]; then
        echo "  $label: destination already exists, skipping ($dst)"
        return 0
    fi

    echo "  $label: $src -> $dst"
    if [[ "$DRY_RUN" == false ]]; then
        mkdir -p "$(dirname "$dst")"
        mv "$src" "$dst"
    fi
}

# macOS-specific: old dirs::data_dir() and dirs::config_dir() paths
if [[ "$OSTYPE" == darwin* ]]; then
    OLD_DATA="$HOME/Library/Application Support/opencode/data"
    OLD_CONFIG="$HOME/Library/Application Support/opencode/config"
    OLD_CACHE="$HOME/Library/Caches/opencode/cache"

    NEW_DATA="${XDG_DATA_HOME:-$HOME/.local/share}/opencode/data"
    NEW_CONFIG="${XDG_CONFIG_HOME:-$HOME/.config}/opencode/config"
    NEW_CACHE="${XDG_CACHE_HOME:-$HOME/.cache}/opencode/cache"

    echo "Migrating opencode-container data to XDG paths on macOS..."
    migrate_dir "$OLD_DATA" "$NEW_DATA" "data"
    migrate_dir "$OLD_CONFIG" "$NEW_CONFIG" "config"
    migrate_dir "$OLD_CACHE" "$NEW_CACHE" "cache"
    echo "Done."
else
    echo "This migration script is for macOS only (platform-native -> XDG)."
    echo "On Linux, data is already in XDG paths. Nothing to do."
fi
