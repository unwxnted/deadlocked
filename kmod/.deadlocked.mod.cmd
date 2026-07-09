savedcmd_deadlocked.mod := printf '%s\n'   deadlocked.o | awk '!x[$$0]++ { print("./"$$0) }' > deadlocked.mod
