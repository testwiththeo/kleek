#!/usr/bin/env bash
set -euo pipefail

REPO="testwiththeo/kleek"
GITHUB_API="https://api.github.com/repos/${REPO}/releases/latest"

echo "==> Installing kleek mouse companion..."

# Check prerequisites
for cmd in curl sudo; do
    if ! command -v "$cmd" >/dev/null 2>&1; then
        echo "Error: required command '$cmd' is not installed." >&2
        exit 1
    fi
done

# Temporary workspace
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

DEB_FILE="${TMP_DIR}/kleek.deb"

# Fetch latest .deb asset download URL
echo "-> Fetching latest release from GitHub (${REPO})..."
RELEASE_JSON="$(curl -fsSL -H "User-Agent: kleek-installer" "$GITHUB_API" 2>/dev/null || true)"

DEB_URL=""
if [ -n "$RELEASE_JSON" ]; then
    DEB_URL="$(echo "$RELEASE_JSON" | grep -o 'https://[^"]*kleek_[^"]*_amd64\.deb' | head -n 1 || true)"
fi

if [ -n "$DEB_URL" ]; then
    echo "-> Downloading package from ${DEB_URL}..."
    curl -fSL -o "$DEB_FILE" "$DEB_URL"
    echo "-> Installing package via dpkg..."
    sudo dpkg -i "$DEB_FILE" || sudo apt-get install -f -y
else
    # Fallback if release asset is not yet published: build from source if cargo exists, or notify user
    echo "! No published release asset found yet."
    if command -v cargo >/dev/null 2>&1 && [ -f "Cargo.toml" ]; then
        echo "-> Building and installing locally with cargo..."
        cargo build --release --workspace
        mkdir -p "$HOME/.cargo/bin"
        cp -f target/release/kleek-daemon "$HOME/.cargo/bin/"
        cp -f target/release/kleekctl "$HOME/.cargo/bin/"
    else
        echo "Error: GitHub release asset not found and local Cargo workspace unavailable." >&2
        echo "Please check https://github.com/${REPO}/releases" >&2
        exit 1
    fi
fi

# Enable and start user systemd service
if command -v systemctl >/dev/null 2>&1; then
    echo "-> Starting user daemon..."
    systemctl --user daemon-reload || true
    systemctl --user enable --now kleek.service 2>/dev/null || true
fi

echo ""
echo "✓ kleek installed successfully!"
echo "Check status with: kleekctl status"
