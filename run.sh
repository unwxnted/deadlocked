#!/usr/bin/env bash

fail() {
	echo "Not a git repository (or any of the parent directories): .git.
Do NOT download the repository as a zip file from github.com!
Please download deadlocked by cloning the Git repository: 'git clone https://github.com/avitran0/deadlocked'"
	exit 1
}

[[ -d '.git' ]] || fail

git config core.hooksPath .hooks

# auto-wrap with sg to ensure the deadlocked group is active
if [ ! -r /dev/deadlocked ] && getent group deadlocked > /dev/null 2>&1; then
	exec sg deadlocked -c "cargo run --release"
fi

cargo run --release
