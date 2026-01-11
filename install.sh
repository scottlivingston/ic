#!/usr/bin/env bash
set -e

IC_REPO="scottlivingston/ic"
IC_DIR="/opt/ic"
IC_USER="${SUDO_USER:-pi}"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

info() { echo -e "${GREEN}[IC]${NC} $1"; }
warn() { echo -e "${YELLOW}[IC]${NC} $1"; }
error() { echo -e "${RED}[IC]${NC} $1"; exit 1; }

is_update() {
  [[ -f "$IC_DIR/ic" ]]
}

check_root() {
  [[ $EUID -ne 0 ]] && error "Run with sudo: curl -sSL ... | sudo bash"
}

check_arch() {
  [[ "$(uname -m)" != "aarch64" ]] && warn "Expected aarch64 (Pi), got $(uname -m)"
}

stop_service() {
  if systemctl is-active --quiet ic.service 2>/dev/null; then
    info "Stopping IC service..."
    systemctl stop ic.service
  fi
}

start_service() {
  info "Starting IC service..."
  systemctl start ic.service
}

install_deps() {
  info "Installing dependencies..."
  apt-get update
  apt-get install -y \
    cage \
    xserver-xorg-core \
    xserver-xorg-video-fbdev \
    xinit \
    libgl1-mesa-dri \
    libgles2 \
    libegl1 \
    mesa-utils
}

download_binary() {
  info "Downloading IC..."
  mkdir -p "$IC_DIR"

  # Get latest release binary URL
  local url=$(curl -s "https://api.github.com/repos/$IC_REPO/releases/latest" \
    | grep "browser_download_url.*ic-aarch64" \
    | cut -d '"' -f 4)

  [[ -z "$url" ]] && error "Could not find release binary"

  curl -L "$url" -o "$IC_DIR/ic"
  chmod +x "$IC_DIR/ic"
  chown -R "$IC_USER:$IC_USER" "$IC_DIR"
}

setup_service() {
  info "Setting up systemd service..."
  cat > /etc/systemd/system/ic.service << EOF
[Unit]
Description=IC Robot Face
After=network.target

[Service]
Type=simple
User=$IC_USER
ExecStart=/usr/bin/cage $IC_DIR/ic
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
EOF
  systemctl daemon-reload
  systemctl enable ic.service
}

setup_autologin() {
  info "Configuring auto-login..."
  mkdir -p /etc/systemd/system/getty@tty1.service.d
  cat > /etc/systemd/system/getty@tty1.service.d/autologin.conf << EOF
[Service]
ExecStart=
ExecStart=-/sbin/agetty --autologin $IC_USER --noclear %I \$TERM
EOF
}

disable_blanking() {
  info "Disabling screen blanking..."
  local cmdline="/boot/cmdline.txt"
  [[ -f "/boot/firmware/cmdline.txt" ]] && cmdline="/boot/firmware/cmdline.txt"
  grep -q "consoleblank=0" "$cmdline" || sed -i 's/$/ consoleblank=0/' "$cmdline"
}

main() {
  echo "=============================="
  echo "  IC Robot Face Installer"
  echo "=============================="

  check_root
  check_arch

  local updating=false
  if is_update; then
    updating=true
    info "Existing installation detected, updating..."
  fi

  stop_service
  install_deps
  download_binary
  setup_service

  if [[ "$updating" == false ]]; then
    setup_autologin
    disable_blanking
  fi

  start_service

  info "Done! IC is now running."
  info "Admin UI: http://<pi-ip>:3000"
}

main "$@"
