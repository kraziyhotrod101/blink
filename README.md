# 👁️ Blink

**Inspired by Howdy, but radically faster—because your face auth shouldn't take longer than typing a password.**

Blink is a lightning-fast Windows Hello™ style facial authentication system for Linux. Written entirely in **Rust**, it utilizes OpenCV and a persistent daemon architecture to keep the models loaded in memory, meaning face-checks happen almost instantly without heavy initialization delays.

---

## ✨ Features
* **Blazing Fast**: Uses a background daemon (`blinkd`) so the OpenCV cascade and LBPH face models are always pre-loaded.
* **Memory Safe**: Written in Rust.
* **IR Camera Support**: Perfect for environments with varying lighting (relies on Infrared capture where available).
* **Drop-in PAM Module**: Plugs directly into `sudo`, `su`, or your lock screen.

---

## 🛠️ Prerequisites

You will need an IR-capable webcam (or a standard webcam, though you may need to update the video device path in the source code).

You also need the following dependencies installed on your system:
```bash
# Ubuntu/Debian example
sudo apt update
sudo apt install -y build-essential curl pkg-config libpam0g-dev libopencv-dev clang
```

Ensure you have [Rust and Cargo](https://rustup.rs/) installed:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

---

## 🚀 Installation

Blink comes with an automated installation script that handles compiling the Rust binaries, moving them to the correct system paths, and setting up the systemd background daemon.

1. **Clone the repository:**
   ```bash
   git clone https://github.com/yourusername/blink.git
   cd blink
   ```

2. **Run the installer:**
   ```bash
   sudo ./install.sh
   ```
   *This script does the following:*
   * *Compiles the release binaries (`blinkd`, `blink-enroll`, and `libpam_blink.so`)*
   * *Copies the PAM module to `/lib/x86_64-linux-gnu/security/`*
   * *Installs binaries to `/usr/local/bin/`*
   * *Sets up and starts the `blinkd` systemd service*

---

## 📸 Enrollment

Before Blink can recognize you, you need to teach it your face.

1. Ensure the directory for the model exists:
   ```bash
   sudo mkdir -p /etc/blink
   ```
2. Run the enrollment tool and look directly into your camera:
   ```bash
   sudo blink-enroll
   ```
   *Note: This will capture multiple frames to train the LBPH Face Recognizer and save the model to `/etc/blink/blink_model.yml`.*

3. Restart the background daemon to load your newly captured face model:
   ```bash
   sudo systemctl restart blinkd
   ```

---

## 🔐 Enabling PAM (sudo)

To actually use Blink for authentication (e.g., when running `sudo`), you need to insert it into your PAM configuration.

1. Open the `/etc/pam.d/sudo` file:
   ```bash
   sudo nano /etc/pam.d/sudo
   ```

2. Add the following line **at the very top** of the file:
   ```text
   auth sufficient pam_blink.so
   ```

Now, try running a `sudo` command in a new terminal window. You should see the IR camera flash and log you in automatically if it recognizes your face!

---

## 🔧 Troubleshooting

If authentication is failing or timing out falling back to the password prompt, check the daemon logs:
```bash
sudo journalctl -u blinkd -f
```

Verify the daemon is actually running:
```bash
systemctl status blinkd
```