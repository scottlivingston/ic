#!/bin/bash
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

info() { echo -e "${GREEN}[INFO]${NC} $1"; }
warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
error() { echo -e "${RED}[ERROR]${NC} $1"; exit 1; }

# Must run as root
if [[ $EUID -ne 0 ]]; then
    error "This script must be run with sudo"
fi

# Get the actual user (not root)
KIOSK_USER="${SUDO_USER:-$USER}"
KIOSK_HOME=$(eval echo ~$KIOSK_USER)
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
IC_BINARY="$SCRIPT_DIR/target/release/ic"

if [[ ! -f "$IC_BINARY" ]]; then
    error "IC binary not found at $IC_BINARY. Run ./setup.sh first."
fi

info "Setting up kiosk mode for user: $KIOSK_USER"

# Install minimal X packages
info "Installing X11 packages..."
apt-get update
apt-get install -y --no-install-recommends \
    xserver-xorg \
    x11-xserver-utils \
    xinit \
    openbox

# Configure auto-login
info "Configuring auto-login..."
mkdir -p /etc/systemd/system/getty@tty1.service.d
cat > /etc/systemd/system/getty@tty1.service.d/autologin.conf << EOF
[Service]
ExecStart=
ExecStart=-/sbin/agetty --autologin $KIOSK_USER --noclear %I \$TERM
EOF

# Create .xinitrc
info "Creating .xinitrc..."
cat > "$KIOSK_HOME/.xinitrc" << EOF
#!/bin/sh

# Disable screen blanking and power management
xset s off
xset s noblank
xset -dpms

# Hide cursor after 1 second of inactivity
# (requires unclutter package, optional)
if command -v unclutter &> /dev/null; then
    unclutter -idle 1 &
fi

# Start openbox window manager
openbox &

# Wait for openbox to start
sleep 1

# Run the IC app
exec $IC_BINARY
EOF
chown "$KIOSK_USER:$KIOSK_USER" "$KIOSK_HOME/.xinitrc"
chmod +x "$KIOSK_HOME/.xinitrc"

# Configure auto-start X on login
info "Configuring auto-start X..."
BASH_PROFILE="$KIOSK_HOME/.bash_profile"
STARTX_LINE='[[ -z $DISPLAY && $XDG_VTNR -eq 1 ]] && startx'

if [[ -f "$BASH_PROFILE" ]]; then
    if ! grep -q "startx" "$BASH_PROFILE"; then
        echo "$STARTX_LINE" >> "$BASH_PROFILE"
    fi
else
    echo "$STARTX_LINE" > "$BASH_PROFILE"
    chown "$KIOSK_USER:$KIOSK_USER" "$BASH_PROFILE"
fi

# Disable console blanking in boot config
info "Disabling console blanking..."
CMDLINE="/boot/cmdline.txt"
if [[ -f "$CMDLINE" ]]; then
    if ! grep -q "consoleblank=0" "$CMDLINE"; then
        sed -i 's/$/ consoleblank=0/' "$CMDLINE"
    fi
else
    # Newer Pi OS uses /boot/firmware/cmdline.txt
    CMDLINE="/boot/firmware/cmdline.txt"
    if [[ -f "$CMDLINE" ]] && ! grep -q "consoleblank=0" "$CMDLINE"; then
        sed -i 's/$/ consoleblank=0/' "$CMDLINE"
    fi
fi

info "Kiosk setup complete!"
info "Reboot to start in kiosk mode: sudo reboot"
