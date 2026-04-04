#!/bin/bash

# Exit immediately if a command exits with a non-zero status
set -e

echo "🚀 Starting Blink Installation..."

# 1. Check for Root Privileges
if [ "$EUID" -ne 0 ]; then
  echo "❌ Please run this script with sudo: sudo ./install.sh"
  exit 1
fi

echo "👋 Welcome to the Blink setup!"
echo "We need to find your IR camera. Installing v4l-utils if missing..."
apt-get install -y v4l-utils > /dev/null 2>&1 || true

echo ""
echo "📸 Available Video Devices:"
v4l2-ctl --list-devices
echo ""
read -p "🎯 Enter the exact path to your IR camera (e.g., /dev/video0 or /dev/v4l/by-path/...): " IR_DEVICE

mkdir -p /etc/blink
echo "$IR_DEVICE" > /etc/blink/device
chmod 644 /etc/blink/device
echo "✅ Camera path saved!"

# 2. Build the Project
echo "🔨 Compiling Rust binaries..."
# We run cargo as the original user so root doesn't take ownership of your target/ folder
sudo -u $SUDO_USER cargo build --release

# 3. Install the PAM Module
echo "📦 Deploying the PAM client..."
cp target/release/libpam_blink.so /lib/x86_64-linux-gnu/security/pam_blink.so
chown root:root /lib/x86_64-linux-gnu/security/pam_blink.so
chmod 755 /lib/x86_64-linux-gnu/security/pam_blink.so
# 4. Install the Daemon
echo "⚙️ Deploying the Blink Daemon..."
# Stop the daemon if it's already running to prevent "Text file busy" error
systemctl stop blinkd 2>/dev/null || true

# FIXED: Now pointing to the correctly named blinkd binary
cp target/release/blinkd /usr/local/bin/blinkd
chown root:root /usr/local/bin/blinkd
chmod 755 /usr/local/bin/blinkd

# 4.5 Install the Enrollment Tool
echo "📸 Deploying the Enrollment Tool..."
cp target/release/enroll /usr/local/bin/blink-enroll
chown root:root /usr/local/bin/blink-enroll
chmod 755 /usr/local/bin/blink-enroll

# 5. Create the Systemd Service
echo "🔌 Registering background service..."
cat << 'EOF' > /etc/systemd/system/blinkd.service
[Unit]
Description=Blink IR Facial Recognition Daemon
After=multi-user.target
Requires=systemd-udevd.service

[Service]
Type=simple
ExecStart=/usr/local/bin/blinkd
Restart=on-failure
RestartSec=5
User=root

[Install]
WantedBy=multi-user.target
EOF

# 6. Enable and Start the Service
echo "🔄 Reloading systemd..."
systemctl daemon-reload
systemctl enable blinkd
systemctl restart blinkd

# 7. Configure PAM automatically
echo "🔐 Configuring PAM for system authentication..."

# List of common PAM files for sudo, su, graphical password prompts, and lock screens/login managers
PAM_FILES=(
    "/etc/pam.d/sudo"
    "/etc/pam.d/su"
    "/etc/pam.d/gdm-password"     # GNOME/Pop!_OS login and lock screen
    "/etc/pam.d/lightdm"          # LightDM login screen
    "/etc/pam.d/sddm"             # KDE/Plasma login screen
    "/etc/pam.d/polkit-1"         # GUI admin authentication prompts
    "/etc/pam.d/gnome-screensaver" # Some older GNOME lock screens
    "/etc/pam.d/kde"              # KDE lock screen
)

for pam_file in "${PAM_FILES[@]}"; do
    if [ -f "$pam_file" ]; then
        if ! grep -q "pam_blink.so" "$pam_file"; then
            sed -i '1i auth sufficient pam_blink.so' "$pam_file"
            echo "✅ Enabled for $(basename "$pam_file")"
        else
            echo "ℹ️ Already enabled for $(basename "$pam_file")"
        fi
    fi
done

echo ""
echo "✅ Blink is fully installed and the daemon is running!"
echo "✨ Next steps: Run 'sudo blink-enroll' to capture your face!"