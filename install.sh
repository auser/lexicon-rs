#!/usr/bin/env bash
set -euo pipefail

REPO="auser/lexicon-rs"
BINARY="lexicon"
CRATE="lexicon-rs"

# Detect OS and architecture
detect_platform() {
  local os arch
  os="$(uname -s)"
  arch="$(uname -m)"

  case "$os" in
    Linux)  os="unknown-linux-gnu" ;;
    Darwin) os="apple-darwin" ;;
    *)      echo "Unsupported OS: $os" >&2; return 1 ;;
  esac

  case "$arch" in
    x86_64)  arch="x86_64" ;;
    aarch64|arm64) arch="aarch64" ;;
    *)       echo "Unsupported architecture: $arch" >&2; return 1 ;;
  esac

  echo "${arch}-${os}"
}

# Install from GitHub release
install_from_release() {
  local platform="$1"
  local version="${2:-latest}"
  local url

  if [ "$version" = "latest" ]; then
    url="https://github.com/${REPO}/releases/latest/download/${BINARY}-${platform}.tar.gz"
  else
    url="https://github.com/${REPO}/releases/download/${version}/${BINARY}-${platform}.tar.gz"
  fi

  local tmpdir
  tmpdir="$(mktemp -d)"
  trap 'rm -rf "$tmpdir"' EXIT

  echo "Downloading ${BINARY} for ${platform}..."
  if curl -fsSL "$url" -o "${tmpdir}/${BINARY}.tar.gz" 2>/dev/null; then
    tar -xzf "${tmpdir}/${BINARY}.tar.gz" -C "$tmpdir"

    local install_dir="${INSTALL_DIR:-/usr/local/bin}"
    if [ -w "$install_dir" ]; then
      mv "${tmpdir}/${BINARY}" "${install_dir}/${BINARY}"
    else
      sudo mv "${tmpdir}/${BINARY}" "${install_dir}/${BINARY}"
    fi

    chmod +x "${install_dir}/${BINARY}"
    echo "Installed ${BINARY} to ${install_dir}/${BINARY}"
    return 0
  fi

  return 1
}

# Install via cargo
install_from_cargo() {
  if ! command -v cargo &>/dev/null; then
    echo "cargo not found. Install Rust first: https://rustup.rs" >&2
    exit 1
  fi

  echo "Installing ${CRATE} via cargo..."
  cargo install "$CRATE"
  echo "Installed ${BINARY} via cargo install"
}

main() {
  echo "Installing ${BINARY}..."

  local platform
  if platform="$(detect_platform)"; then
    if install_from_release "$platform" "${1:-latest}"; then
      return
    fi
    echo "No prebuilt binary available, falling back to cargo install..."
  else
    echo "Could not detect platform, falling back to cargo install..."
  fi

  install_from_cargo
}

main "$@"
