#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Fetch the MediaPipe Selfie Segmentation ONNX model used by StyledCamera.

Default output:
  ./model/mediapipe_selfie_segmentation.onnx

Usage:
  ./scripts/fetch-model.sh [--out PATH] [--url URL] [--sha256 HEX] [--force]

Options:
  --out PATH     Output file path (default: ./model/mediapipe_selfie_segmentation.onnx)
  --url URL      Model download URL
  --sha256 HEX   Expected SHA-256 (optional)
  --force        Re-download even if output file exists

Environment variables (alternatives to flags):
  MODEL_OUT, MODEL_URL, MODEL_SHA256
EOF
}

force=0
out_path="${MODEL_OUT:-./model/mediapipe_selfie_segmentation.onnx}"
url="${MODEL_URL:-https://huggingface.co/onnx-community/mediapipe_selfie_segmentation/resolve/main/onnx/model.onnx}"
expected_sha256="${MODEL_SHA256:-}"

while [[ $# -gt 0 ]]; do
  case "$1" in
    -h|--help) usage; exit 0 ;;
    --force) force=1; shift ;;
    --out) out_path="${2:-}"; shift 2 ;;
    --url) url="${2:-}"; shift 2 ;;
    --sha256) expected_sha256="${2:-}"; shift 2 ;;
    *) echo "Unknown arg: $1" >&2; usage >&2; exit 2 ;;
  esac
done

if [[ -z "${out_path}" ]]; then
  echo "--out cannot be empty" >&2
  exit 2
fi
if [[ -z "${url}" ]]; then
  echo "--url cannot be empty" >&2
  exit 2
fi

out_dir="$(cd "$(dirname "${out_path}")" && pwd)"
out_file="$(basename "${out_path}")"
mkdir -p "${out_dir}"

final_path="${out_dir}/${out_file}"
if [[ -f "${final_path}" && "${force}" -ne 1 ]]; then
  echo "Already exists: ${final_path}"
  echo "Use --force to re-download."
  exit 0
fi

tmp="$(mktemp "${out_dir}/.${out_file}.tmp.XXXXXX")"
cleanup() { rm -f "${tmp}"; }
trap cleanup EXIT

echo "Downloading model:"
echo "  ${url}"
echo "To:"
echo "  ${final_path}"

curl -fL --retry 3 --retry-delay 1 -o "${tmp}" "${url}"

if [[ -n "${expected_sha256}" ]]; then
  actual="$(shasum -a 256 "${tmp}" | awk '{print $1}')"
  if [[ "${actual}" != "${expected_sha256}" ]]; then
    echo "SHA-256 mismatch:" >&2
    echo "  expected: ${expected_sha256}" >&2
    echo "  actual:   ${actual}" >&2
    exit 1
  fi
fi

mv -f "${tmp}" "${final_path}"
trap - EXIT

echo "Done:"
echo "  ${final_path}"

