#!/usr/bin/env bash

fail() {
	echo "Not a git repository (or any of the parent directories): .git.
Do NOT download the repository as a zip file from github.com!
Please download deadlocked by cloning the Git repository: 'git clone https://github.com/avitran0/deadlocked'"
	exit 1
}

[[ -d '.git' ]] || fail

git config core.hooksPath .hooks

# ---------- kernel module ----------
# Try to load the module if not already loaded (fails silently if already loaded)
sudo insmod kmod/iomem_rw.ko 2>/dev/null || true

# ---------- uinput group ----------
if [ -c /dev/uinput ] && ! groups | grep -q uinput; then
	if getent group uinput > /dev/null 2>&1; then
		exec sg uinput -c "cargo run --release"
	fi
fi

cargo run --release
