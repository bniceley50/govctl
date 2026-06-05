#!/usr/bin/env bash
# Install govctl into $INSTALL_DIR. Tries a prebuilt release binary first (fast path),
# then falls back to building from source. Designed to be testable outside CI:
#   GOVCTL_REPO    owner/repo or a full git URL (default: bniceley50/govctl)
#   GOVCTL_VERSION git ref / release tag       (default: v0.3.4)
#   INSTALL_DIR    where to place the govctl binary
set -euo pipefail

GOVCTL_REPO="${GOVCTL_REPO:-bniceley50/govctl}"
GOVCTL_VERSION="${GOVCTL_VERSION:-v0.3.4}"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"
mkdir -p "$INSTALL_DIR"

# Normalize repo into a clone URL and an owner/repo slug.
if [[ "$GOVCTL_REPO" == http*://* || "$GOVCTL_REPO" == git@* || "$GOVCTL_REPO" == file://* ]]; then
  REPO_URL="$GOVCTL_REPO"
  REPO_SLUG="$(basename "${GOVCTL_REPO%.git}")"
else
  REPO_URL="https://github.com/${GOVCTL_REPO}"
  REPO_SLUG="$GOVCTL_REPO"
fi

detect_target() {
  local os arch
  os="$(uname -s)"; arch="$(uname -m)"
  case "$os" in
    Linux)  case "$arch" in x86_64) echo "x86_64-unknown-linux-gnu";; aarch64|arm64) echo "aarch64-unknown-linux-gnu";; *) echo "";; esac;;
    Darwin) case "$arch" in x86_64) echo "x86_64-apple-darwin";; arm64) echo "aarch64-apple-darwin";; *) echo "";; esac;;
    *) echo "";;
  esac
}

try_binary() {
  local target tarball url
  target="$(detect_target)"
  [ -z "$target" ] && return 1
  tarball="govctl-${GOVCTL_VERSION}-${target}.tar.gz"
  url="https://github.com/${REPO_SLUG}/releases/download/${GOVCTL_VERSION}/${tarball}"
  echo "govctl: trying prebuilt binary $url"
  local tmp; tmp="$(mktemp -d)"
  if curl -fsSL "$url" -o "$tmp/$tarball" 2>/dev/null; then
    tar -xzf "$tmp/$tarball" -C "$tmp"
    if [ -f "$tmp/govctl" ]; then
      install -m 0755 "$tmp/govctl" "$INSTALL_DIR/govctl"
      echo "govctl: installed prebuilt binary into $INSTALL_DIR"
      return 0
    fi
  fi
  echo "govctl: no prebuilt binary available, will build from source"
  return 1
}

build_from_source() {
  command -v cargo >/dev/null 2>&1 || { echo "govctl: cargo not found; cannot build from source" >&2; return 1; }
  local tmp; tmp="$(mktemp -d)"
  echo "govctl: cloning $REPO_URL @ $GOVCTL_VERSION"
  git clone --depth 1 --branch "$GOVCTL_VERSION" "$REPO_URL" "$tmp/src" 2>/dev/null \
    || git clone "$REPO_URL" "$tmp/src"
  ( cd "$tmp/src" && cargo build --release )
  install -m 0755 "$tmp/src/target/release/govctl" "$INSTALL_DIR/govctl"
  echo "govctl: installed from source into $INSTALL_DIR"
}

if ! try_binary; then
  build_from_source
fi

"$INSTALL_DIR/govctl" --version
