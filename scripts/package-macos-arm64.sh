#!/usr/bin/env bash
set -euo pipefail

PLUGIN_NAME="StyledCamera"
CRATE_NAME="styledcamera"
CRATE_LIB_DEFAULT="libstyledcamera.dylib"
DIST_ROOT_DEFAULT="./dist"
DIST_MACOS_SUBDIR_DEFAULT="macos"
MODEL_SRC_DEFAULT="./model/mediapipe_selfie_segmentation.onnx"
MODEL_DEST_NAME_DEFAULT="selfie_segmentation.onnx"
EFFECTS_DIR_DEFAULT="./data/effects"

usage() {
  cat <<'EOF'
Package StyledCamera for macOS arm64 into ./dist/ (plugin bundle + zip).

What it does:
  - Builds the Rust plugin binary (cdylib)
  - Creates a .plugin bundle in dist
  - Bundles OBS .effect shaders into Contents/Resources/
  - Bundles the ONNX model into Contents/Resources/models/
  - Bundles third_party/NOTICE.md into Contents/Resources/
  - Bundles libonnxruntime.dylib (either provided or downloaded)
  - Creates a zip suitable for distribution

Usage:
  ./scripts/package-macos-arm64.sh [options]

Options:
  --out-dir PATH                Dist root directory (default: ./dist)
  --dist-subdir NAME            Subdir under out-dir for macOS bundle (default: macos)
  --model PATH                  Model file (default: ./model/mediapipe_selfie_segmentation.onnx)
  --model-dest-name NAME        Name inside the bundle (default: selfie_segmentation.onnx)
  --effects-dir PATH            Directory with .effect files (default: ./data/effects)

  --onnxruntime PATH            Path to libonnxruntime.dylib (preferred)
  --download-onnxruntime        Download ONNX Runtime dylib from GitHub releases
  --onnxruntime-version VER     Version for download (or set ONNXRUNTIME_VERSION); if omitted, uses latest

  --skip-zip                    Do not create the zip

Environment variables:
  ONNXRUNTIME_DYLIB             Same as --onnxruntime
  ONNXRUNTIME_VERSION           Default version for --download-onnxruntime

Outputs:
  dist/macos/StyledCamera.plugin
  dist/StyledCamera-macos-arm64.zip
EOF
}

die() {
  echo "Error: $*" >&2
  exit 1
}

require_cmd() {
  command -v "$1" >/dev/null 2>&1 || die "Missing required command: $1"
}

copy_dir() {
  local src="$1"
  local dst="$2"
  rm -rf "${dst}"
  mkdir -p "$(dirname "${dst}")"
  if command -v rsync >/dev/null 2>&1; then
    rsync -a "${src}/" "${dst}/"
  else
    cp -R "${src}" "${dst}"
  fi
}

github_release_json() {
  local url="$1"
  local body
  body="$(curl -fsSL \
    -H "Accept: application/vnd.github+json" \
    -H "User-Agent: styledcamera-packager" \
    "${url}" || true)"
  [[ -n "${body}" ]] || die "GitHub API returned an empty response (${url}). Try again later or provide --onnxruntime PATH."
  if [[ "${body:0:1}" != "{" ]]; then
    echo "GitHub API unexpected response (first 200 bytes):" >&2
    echo "${body}" | head -c 200 >&2
    echo "" >&2
    die "GitHub API did not return JSON (${url}). Try again later or provide --onnxruntime PATH."
  fi
  printf "%s" "${body}"
}

resolve_latest_onnxruntime_release() {
  github_release_json "https://api.github.com/repos/microsoft/onnxruntime/releases/latest" \
    | /usr/bin/python3 - <<'PY'
import json, sys
data = json.load(sys.stdin)
tag = data.get("tag_name") or ""
if tag.startswith("v"):
  tag = tag[1:]
if not tag:
  raise SystemExit("failed to resolve latest onnxruntime tag")
print(tag)
PY
}

resolve_onnxruntime_osx_arm64_tgz_url() {
  local version="$1"
  github_release_json "https://api.github.com/repos/microsoft/onnxruntime/releases/tags/v${version}" \
    | /usr/bin/python3 - <<'PY'
import json, sys
data = json.load(sys.stdin)
assets = data.get("assets") or []

def ok(name: str) -> bool:
  name_l = name.lower()
  if "osx" not in name_l:
    return False
  if "arm64" not in name_l:
    return False
  if not (name_l.endswith(".tgz") or name_l.endswith(".tar.gz")):
    return False
  # ignore training / gpu packages if present
  return "onnxruntime" in name_l

matches = [a for a in assets if ok(a.get("name") or "")]
if not matches:
  raise SystemExit("no osx-arm64 .tgz asset found in release")
print(matches[0]["browser_download_url"])
PY
}

download_onnxruntime_dylib() {
  local version="$1"
  local url="$2"
  local tmpdir="$3"

  require_cmd curl
  require_cmd tar
  require_cmd find

  local archive="${tmpdir}/onnxruntime-osx-arm64.tgz"
  echo "Downloading ONNX Runtime (macOS arm64) ${version}:" >&2
  echo "  ${url}" >&2
  curl -fL --retry 3 --retry-delay 1 -o "${archive}" "${url}"

  tar -xzf "${archive}" -C "${tmpdir}"

  local dylib
  dylib="$(find "${tmpdir}" -name 'libonnxruntime*.dylib' -type f -print -quit)"
  if [[ -z "${dylib}" ]]; then
    echo "Could not find libonnxruntime*.dylib inside the downloaded archive." >&2
    echo "Archive contents (first 50 files):" >&2
    find "${tmpdir}" -type f | head -n 50 >&2
    die "ONNX Runtime archive layout changed; download libonnxruntime.dylib manually and pass --onnxruntime PATH."
  fi

  echo "${dylib}"
}

write_infoplist() {
  local bundle="$1"
  local plist="${bundle}/Contents/Info.plist"
  cat >"${plist}" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleDevelopmentRegion</key>
  <string>en</string>
  <key>CFBundleDisplayName</key>
  <string>${PLUGIN_NAME}</string>
  <key>CFBundleExecutable</key>
  <string>${PLUGIN_NAME}</string>
  <key>CFBundleIdentifier</key>
  <string>com.styledcamera.${PLUGIN_NAME}</string>
  <key>CFBundleInfoDictionaryVersion</key>
  <string>6.0</string>
  <key>CFBundleName</key>
  <string>${PLUGIN_NAME}</string>
  <key>CFBundlePackageType</key>
  <string>BNDL</string>
  <key>CFBundleShortVersionString</key>
  <string>0.1.0</string>
  <key>CFBundleVersion</key>
  <string>0.1.0</string>
</dict>
</plist>
EOF
}

bundle_notice() {
  local bundle="$1"
  local notice_src="$2"

  local res_dir="${bundle}/Contents/Resources"
  mkdir -p "${res_dir}/third_party"
  cp -f "${notice_src}" "${res_dir}/NOTICE.md"
  cp -f "${notice_src}" "${res_dir}/third_party/NOTICE.md"
}

bundle_effects() {
  local bundle="$1"
  local effects_dir="$2"

  [[ -d "${effects_dir}" ]] || die "Effects dir not found: ${effects_dir}"

  local res_dir="${bundle}/Contents/Resources"
  mkdir -p "${res_dir}"

  local n=0
  while IFS= read -r -d '' f; do
    cp -f "${f}" "${res_dir}/"
    n=$((n+1))
  done < <(find "${effects_dir}" -maxdepth 1 -type f -name "*.effect" -print0)

  [[ "${n}" -gt 0 ]] || die "No .effect files found in ${effects_dir}"
}

bundle_model() {
  local bundle="$1"
  local model_src="$2"
  local model_dest_name="$3"

  local models_dir="${bundle}/Contents/Resources/models"
  mkdir -p "${models_dir}"
  cp -f "${model_src}" "${models_dir}/${model_dest_name}"
}

bundle_onnxruntime() {
  local bundle="$1"
  local dylib_src="$2"

  local fw_dir="${bundle}/Contents/Frameworks"
  mkdir -p "${fw_dir}"
  cp -f "${dylib_src}" "${fw_dir}/libonnxruntime.dylib"
}

dist_root="${DIST_ROOT_DEFAULT}"
dist_subdir="${DIST_MACOS_SUBDIR_DEFAULT}"
model_src="${MODEL_SRC_DEFAULT}"
model_dest_name="${MODEL_DEST_NAME_DEFAULT}"
effects_dir="${EFFECTS_DIR_DEFAULT}"
onnxruntime_dylib="${ONNXRUNTIME_DYLIB:-}"
download_ort=0
ort_version="${ONNXRUNTIME_VERSION:-}"
skip_zip=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    -h|--help) usage; exit 0 ;;
    --out-dir) dist_root="${2:-}"; shift 2 ;;
    --dist-subdir) dist_subdir="${2:-}"; shift 2 ;;
    --model) model_src="${2:-}"; shift 2 ;;
    --model-dest-name) model_dest_name="${2:-}"; shift 2 ;;
    --effects-dir) effects_dir="${2:-}"; shift 2 ;;
    --onnxruntime) onnxruntime_dylib="${2:-}"; shift 2 ;;
    --download-onnxruntime) download_ort=1; shift ;;
    --onnxruntime-version) ort_version="${2:-}"; shift 2 ;;
    --skip-zip) skip_zip=1; shift ;;
    *) die "Unknown arg: $1 (use --help)" ;;
  esac
done

if [[ "$(uname -s)" != "Darwin" ]]; then
  die "This script is intended to run on macOS (uname -s == Darwin)."
fi

[[ -f "${model_src}" ]] || die "Model not found: ${model_src} (run ./scripts/fetch-model.sh)"
[[ -f "./third_party/NOTICE.md" ]] || die "Missing ./third_party/NOTICE.md (expected in repo)"

dest_dir="${dist_root}/${dist_subdir}"
dest_bundle="${dest_dir}/${PLUGIN_NAME}.plugin"
zip_path="${dist_root}/${PLUGIN_NAME}-macos-arm64.zip"

echo "Building Rust plugin (${CRATE_NAME})..."
require_cmd cargo
cargo build -p "${CRATE_NAME}" --release --target aarch64-apple-darwin

crate_lib="./target/aarch64-apple-darwin/release/${CRATE_LIB_DEFAULT}"
[[ -f "${crate_lib}" ]] || die "Built library not found: ${crate_lib}"

echo "Creating bundle:"
echo "  ${dest_bundle}"
rm -rf "${dest_bundle}"
mkdir -p "${dest_bundle}/Contents/MacOS"
mkdir -p "${dest_bundle}/Contents/Resources"

cp -f "${crate_lib}" "${dest_bundle}/Contents/MacOS/${PLUGIN_NAME}"
chmod +x "${dest_bundle}/Contents/MacOS/${PLUGIN_NAME}"

write_infoplist "${dest_bundle}"
echo "BNDL????" > "${dest_bundle}/Contents/PkgInfo"

echo "Bundling NOTICE.md..."
bundle_notice "${dest_bundle}" "./third_party/NOTICE.md"

echo "Bundling .effect shaders..."
bundle_effects "${dest_bundle}" "${effects_dir}"

echo "Bundling model..."
bundle_model "${dest_bundle}" "${model_src}" "${model_dest_name}"

tmpdir=""
cleanup() {
  [[ -n "${tmpdir}" ]] && rm -rf "${tmpdir}" || true
}
trap cleanup EXIT

if [[ -z "${onnxruntime_dylib}" ]]; then
  if [[ "${download_ort}" -eq 1 ]]; then
    if [[ -z "${ort_version}" ]]; then
      # Pin by default to a version compatible with the `ort` crate (ORT 1.23+).
      ort_version="1.23.0"
    fi
    tmpdir="$(mktemp -d)"
    ort_url="https://github.com/microsoft/onnxruntime/releases/download/v${ort_version}/onnxruntime-osx-arm64-${ort_version}.tgz"
    onnxruntime_dylib="$(download_onnxruntime_dylib "${ort_version}" "${ort_url}" "${tmpdir}")"
  else
    die "libonnxruntime.dylib is required. Provide --onnxruntime PATH, set ONNXRUNTIME_DYLIB, or use --download-onnxruntime."
  fi
fi

[[ -f "${onnxruntime_dylib}" ]] || die "libonnxruntime.dylib not found: ${onnxruntime_dylib}"

echo "Bundling libonnxruntime.dylib..."
bundle_onnxruntime "${dest_bundle}" "${onnxruntime_dylib}"

if [[ "${skip_zip}" -eq 0 ]]; then
  require_cmd ditto
  mkdir -p "${dist_root}"
  rm -f "${zip_path}"
  echo "Creating zip:"
  echo "  ${zip_path}"
  ditto -c -k --sequesterRsrc --keepParent "${dest_bundle}" "${zip_path}"
fi

echo "Done."
echo "Bundle:"
echo "  ${dest_bundle}"
if [[ "${skip_zip}" -eq 0 ]]; then
  echo "Zip:"
  echo "  ${zip_path}"
fi
