#!/usr/bin/env bash

UINPUT_UDEV_RULE="/etc/udev/rules.d/99-uinput.rules"
DEADLOCKED_UDEV_RULE="/etc/udev/rules.d/99-deadlocked.rules"
UINPUT_GROUP="uinput"
DEADLOCKED_GROUP="deadlocked"
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

# ---------- deadlocked kernel module (memory access) ----------
echo 'KERNEL=="deadlocked", MODE="0660", GROUP="deadlocked"' | sudo tee "$DEADLOCKED_UDEV_RULE" > /dev/null
echo "created udev file: $DEADLOCKED_UDEV_RULE"

if ! getent group "$DEADLOCKED_GROUP" > /dev/null; then
    sudo groupadd "$DEADLOCKED_GROUP"
    echo "created group $DEADLOCKED_GROUP"
fi

sudo usermod -aG "$DEADLOCKED_GROUP" "$CURRENT_USER"
echo "added user $CURRENT_USER to group $DEADLOCKED_GROUP"

# build kernel module
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
echo "built kernel module: kmod/deadlocked.ko"

# copy to a standard location
sudo cp kmod/deadlocked.ko /lib/modules/$(uname -r)/extra/
sudo depmod
echo "installed kernel module"

# load the module
sudo modprobe deadlocked 2>/dev/null || sudo insmod kmod/deadlocked.ko
echo "loaded deadlocked kernel module"

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
