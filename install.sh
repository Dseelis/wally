#!/bin/sh
set -eu

REPO="${WALLY_REPO:-Dseelis/wally}"
VERSION="${WALLY_VERSION:-latest}"
INSTALL_DIR="${WALLY_INSTALL_DIR:-$HOME/.local/bin}"

log() {
    printf '%s\n' "$*"
}

die() {
    printf 'Error: %s\n' "$*" >&2
    exit 1
}

command -v curl >/dev/null 2>&1 || die "curl is required."
command -v tar >/dev/null 2>&1 || die "tar is required."
command -v install >/dev/null 2>&1 || die "install is required."

[ "$(uname -s)" = "Linux" ] || die "Wally currently provides Linux binaries."

ARCH="$(uname -m)"

case "$ARCH" in
    x86_64|amd64)
        ARCH="x86_64"
        ;;
    aarch64|arm64)
        ARCH="aarch64"
        ;;
    *)
        die "Unsupported architecture: $(uname -m)"
        ;;
esac

ASSET="wally-${ARCH}-linux.tar.gz"
BASE_URL="https://github.com/${REPO}/releases"

if [ "$VERSION" = "latest" ]; then
    URL="${BASE_URL}/latest/download/${ASSET}"
else
    URL="${BASE_URL}/download/${VERSION}/${ASSET}"
fi

TMP_DIR="$(mktemp -d)"
EXTRACT_DIR="$TMP_DIR/extracted"
ARCHIVE="$TMP_DIR/$ASSET"

cleanup() {
    rm -rf "$TMP_DIR"
}

trap cleanup EXIT INT TERM

log "Installing Wally..."
log "Repository: $REPO"
log "Architecture: $ARCH"
log "Download: $URL"
log ""

mkdir -p "$EXTRACT_DIR"

if ! curl -fL --retry 3 --progress-bar "$URL" -o "$ARCHIVE"; then
    die "Could not download Wally. Check that the GitHub Release and asset exist."
fi

log ""
log "Extracting Wally..."

if ! tar -xzf "$ARCHIVE" -C "$EXTRACT_DIR"; then
    die "Could not extract the release archive."
fi

BINARY="$(find "$EXTRACT_DIR" -type f -name "wally" -print -quit)"

if [ -z "$BINARY" ]; then
    die "The release archive does not contain a 'wally' binary."
fi

mkdir -p "$INSTALL_DIR"

install -Dm755 "$BINARY" "$INSTALL_DIR/wally"

log ""
log "✓ Wally installed successfully!"
log "✓ Location: $INSTALL_DIR/wally"

case ":${PATH}:" in
    *:"$INSTALL_DIR":*)
        log "✓ $INSTALL_DIR is already in PATH."
        ;;
    *)
        log ""
        log "Wally was installed, but $INSTALL_DIR is not in your PATH."
        log ""
        log "Run:"
        log ""
        log "  export PATH=\"\$HOME/.local/bin:\$PATH\""
        log ""
        log "For a permanent setup, add this line to your shell config."
        ;;
esac

if [ -x "$INSTALL_DIR/wally" ]; then
    log ""
    log "Version:"
    "$INSTALL_DIR/wally" --version 2>/dev/null || true
fi

log ""
log "Enjoy Wally! 🖼️"
