#!/usr/bin/env bash
set -euo pipefail

echo "==> Installing kleek mouse companion..."

# 1. Build release binaries
cargo build --release --workspace

# 2. Install binaries to user path (~/.cargo/bin)
mkdir -p "$HOME/.cargo/bin"
cp -f target/release/kleek-daemon "$HOME/.cargo/bin/"
cp -f target/release/kleekctl "$HOME/.cargo/bin/"
echo "✓ Binaries installed to $HOME/.cargo/bin"

# 3. Install systemd user service
mkdir -p "$HOME/.config/systemd/user"
cp -f systemd/kleek.service "$HOME/.config/systemd/user/"
systemctl --user daemon-reload || true
echo "✓ Systemd user service installed to $HOME/.config/systemd/user/kleek.service"

# 4. Prompt for udev rules installation
UDEV_TARGET="/etc/udev/rules.d/99-kleek.rules"
if [ -f "$UDEV_TARGET" ]; then
    echo "✓ Udev rules already present at $UDEV_TARGET"
else
    echo "-> Installing udev rules requires root permissions:"
    if command -v sudo >/dev/null 2>&1; then
        sudo cp udev/99-kleek.rules "$UDEV_TARGET"
        sudo udevadm control --reload-rules && sudo udevadm trigger
        echo "✓ Udev rules installed and reloaded."
    else
        echo "! Please copy udev/99-kleek.rules to /etc/udev/rules.d/ manually as root."
    fi
fi

echo ""
echo "Installation complete!"
echo "Start the daemon now with: systemctl --user start kleek"
echo "Or run kleekctl to verify:  kleekctl status"
