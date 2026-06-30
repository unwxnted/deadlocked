#!/usr/bin/env bash

UINPUT_UDEV_RULE="/etc/udev/rules.d/99-uinput.rules"
UINPUT_GROUP="uinput"
CURRENT_USER=$(whoami)

git config core.hooksPath .hooks

# ---------- uinput (input emulation) ----------
echo 'KERNEL=="uinput", MODE="0660", GROUP="uinput"' | sudo tee "$UINPUT_UDEV_RULE" > /dev/null
echo "created udev file: $UINPUT_UDEV_RULE"

if ! getent group "$UINPUT_GROUP" > /dev/null; then
    sudo groupadd "$UINPUT_GROUP"
    echo "created group $UINPUT_GROUP"
fi

sudo usermod -aG "$UINPUT_GROUP" "$CURRENT_USER"
echo "added user $CURRENT_USER to group $UINPUT_GROUP"

# ---------- kernel module (memory access via ftrace) ----------
if [ ! -d "/lib/modules/$(uname -r)/build" ]; then
    echo "ERROR: kernel headers not found."
    echo "install them with your package manager, e.g.:"
    echo "  apt install linux-headers-\$(uname -r)   (Debian/Ubuntu)"
    echo "  dnf install kernel-devel                 (Fedora)"
    echo "  pacman -S linux-headers                  (Arch)"
    exit 1
fi

make -C kmod
if [ $? -ne 0 ]; then
    echo "ERROR: failed to build kernel module"
    exit 1
fi
echo "built kernel module: kmod/iomem_rw.ko"

# copy to a standard location
sudo mkdir -p /lib/modules/$(uname -r)/extra
sudo cp kmod/iomem_rw.ko /lib/modules/$(uname -r)/extra/
sudo depmod
echo "installed kernel module"

# load the module (fails silently if already loaded)
sudo insmod kmod/iomem_rw.ko 2>/dev/null && echo "loaded kernel module" || echo "kernel module already loaded (or load failed, check with 'sudo dmesg | tail')"

# ---------- udev reload ----------
sudo udevadm control --reload-rules
sudo udevadm trigger
echo "reloaded udev rules"

# ---------- Hyprland overlay rule ----------
if [ "$XDG_CURRENT_DESKTOP" = "Hyprland" ]; then
    echo "detected Hyprland as window manager"

    RULE="windowrule = no_blur 1, match:title ^(deadlocked_overlay)$"
    CONF_FILE="$HOME/.config/hypr/hyprland.conf"

    if grep -Fxq "$RULE" "$CONF_FILE"; then
        echo "deadlocked_overlay windowrule has already been added, skipping"
    else
        echo "$RULE" >> "$CONF_FILE"
        echo "added windowrule to Hyprland"
    fi
fi

echo ""
echo "=== Setup complete ==="
echo "Log out and log back in for the uinput group to take effect."
echo "Then run: ./run.sh"
