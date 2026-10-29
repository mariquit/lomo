#!/bin/sh
# Download the latest lomo release for this platform and run `lomo setup`.
set -eu

repo="${LOMO_REPO:-mariquit/lomo}"
os=$(uname -s)
arch=$(uname -m)

case "$os-$arch" in
  Linux-x86_64) asset=lomo-x86_64-unknown-linux-gnu ;;
  Linux-aarch64 | Linux-arm64) asset=lomo-aarch64-unknown-linux-gnu ;;
  Darwin-x86_64) asset=lomo-x86_64-apple-darwin ;;
  Darwin-arm64) asset=lomo-aarch64-apple-darwin ;;
  MINGW* | MSYS* | CYGWIN*) asset=lomo-x86_64-pc-windows-msvc.exe ;;
  *)
    echo "lomo: unsupported platform $os $arch" >&2
    echo "download a release asset from https://github.com/${repo}/releases/latest and run it with setup" >&2
    exit 1
    ;;
esac

tmpdir=$(mktemp -d)
trap 'rm -rf "$tmpdir"' EXIT
url="https://github.com/${repo}/releases/latest/download/${asset}"
echo "downloading ${url}"
curl -fsSL -o "$tmpdir/lomo" "$url"
curl -fsSL -o "$tmpdir/lomo.sha256" "${url}.sha256"
if command -v sha256sum >/dev/null 2>&1; then
  actual=$(sha256sum "$tmpdir/lomo" | awk '{print $1}')
elif command -v shasum >/dev/null 2>&1; then
  actual=$(shasum -a 256 "$tmpdir/lomo" | awk '{print $1}')
else
  echo "lomo: need sha256sum or shasum to verify ${asset}" >&2
  exit 1
fi
expected=$(awk '{print $1}' "$tmpdir/lomo.sha256")
if [ -z "$expected" ] || [ "$expected" != "$actual" ]; then
  echo "lomo: checksum mismatch for ${asset}" >&2
  echo "expected ${expected:-<empty>}" >&2
  echo "actual   ${actual}" >&2
  exit 1
fi
echo "checksum ok"
chmod +x "$tmpdir/lomo"

home="${HOME:-${USERPROFILE:-}}"
cargo_home="${CARGO_HOME:-$home/.cargo}"
bin_dir=""
old_ifs=$IFS
IFS=':'
for candidate in "$home/.local/bin" "$cargo_home/bin" "$home/.bun/bin"; do
  candidate=${candidate%/}
  for dir in $PATH; do
    dir=${dir%/}
    if [ "$dir" = "$candidate" ]; then
      bin_dir=$candidate
      break 2
    fi
  done
done
IFS=$old_ifs
if [ -z "$bin_dir" ]; then
  bin_dir="$home/.local/bin"
fi
mkdir -p "$bin_dir"
name=lomo
case "$asset" in
  *.exe) name=lomo.exe ;;
esac
cp "$tmpdir/lomo" "$bin_dir/$name"
chmod +x "$bin_dir/$name"
for stale in "$home/.local/bin/$name" "$cargo_home/bin/$name" "$home/.bun/bin/$name"; do
  if [ -f "$stale" ] && [ "$stale" != "$bin_dir/$name" ]; then
    cp "$tmpdir/lomo" "$stale"
    chmod +x "$stale"
  fi
done
LOMO_BIN_DIR="$bin_dir" "$bin_dir/$name" setup
