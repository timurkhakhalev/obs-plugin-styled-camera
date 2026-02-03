#!/usr/bin/env bash
set -euo pipefail

PLUGIN_BUNDLE_SRC="${1:-}"
if [[ -z "${PLUGIN_BUNDLE_SRC}" ]]; then
  if [[ -d "./StyledCamera.plugin" ]]; then
    PLUGIN_BUNDLE_SRC="./StyledCamera.plugin"
  elif [[ -d "./dist/macos/StyledCamera.plugin" ]]; then
    PLUGIN_BUNDLE_SRC="./dist/macos/StyledCamera.plugin"
  else
    echo "Usage: $0 /path/to/StyledCamera.plugin"
    echo "Expected to find ./StyledCamera.plugin or ./dist/macos/StyledCamera.plugin"
    exit 2
  fi
fi

if [[ ! -d "${PLUGIN_BUNDLE_SRC}" ]]; then
  echo "Plugin bundle not found: ${PLUGIN_BUNDLE_SRC}"
  exit 2
fi

OBS_USER_PLUGINS_DIR="${HOME}/Library/Application Support/obs-studio/plugins"
TARGET_BUNDLE_DIR="${OBS_USER_PLUGINS_DIR}/StyledCamera.plugin"

echo "Installing StyledCamera to:"
echo "  ${TARGET_BUNDLE_DIR}"

mkdir -p "${OBS_USER_PLUGINS_DIR}"
rm -rf "${TARGET_BUNDLE_DIR}"

if command -v rsync >/dev/null 2>&1; then
  rsync -a "${PLUGIN_BUNDLE_SRC}/" "${TARGET_BUNDLE_DIR}/"
else
  cp -R "${PLUGIN_BUNDLE_SRC}" "${TARGET_BUNDLE_DIR}"
fi

# If the bundle was downloaded from the internet, it may be quarantined by Gatekeeper.
# This is best-effort; it is OK if xattr is not present or returns an error.
xattr -dr com.apple.quarantine "${TARGET_BUNDLE_DIR}" >/dev/null 2>&1 || true

echo "Done."
echo "Next: start OBS and verify the plugin loads (check Help -> Log Files -> View Current Log)."

