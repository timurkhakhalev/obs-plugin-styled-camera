#!/usr/bin/env bash
set -euo pipefail

TARGET_BUNDLE_DIR="${HOME}/Library/Application Support/obs-studio/plugins/StyledCamera.plugin"

if [[ -d "${TARGET_BUNDLE_DIR}" ]]; then
  echo "Removing:"
  echo "  ${TARGET_BUNDLE_DIR}"
  rm -rf "${TARGET_BUNDLE_DIR}"
  echo "Done."
else
  echo "Not installed (missing): ${TARGET_BUNDLE_DIR}"
fi

