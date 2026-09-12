#!/bin/sh
set -eu

# Change this to your GitHub repository: owner/repository
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
trap 'rm -rf "$TMP_DIR"' EXIT

ARCHIVE="$TMP_DIR/$ASSET"

log "Installing Wally..."
log "Repository: $REPO"
log "Architecture: $ARCH"
log "Download: $URL"

if ! curl -fL --retry 3 --progress-bar "$URL" -o "$ARCHIVE"; then
    die "Could not download Wally. Check that the GitHub Release and asset exist."
fi

mkdir -p "$INSTALL_DIR"

tar -xzf "$ARCHIVE" -C "$TMP_DIR"

if [ ! -f "$TMP_DIR/wally" ]; then
    die "The release archive does not contain a 'wally' binary."
fi

install -Dm755 "$TMP_DIR/wally" "$INSTALL_DIR/wally"

log ""
log "✓ Wally installed to: $INSTALL_DIR/wally"

case ":${PATH}:" in
    *:"$INSTALL_DIR":*)
        log "✓ $INSTALL_DIR is already in PATH."
        ;;
    *)
        log ""
        log "Add Wally to PATH with:"
        log ""
        log "  export PATH=\"\$HOME/.local/bin:\$PATH\""
        log ""
        log "For a permanent setup, add that line to ~/.bashrc or ~/.zshrc."
        ;;
esac

if command -v "$INSTALL_DIR/wally" >/dev/null 2>&1; then
    log ""
    "$INSTALL_DIR/wally" --version 2>/dev/null || true
fi
