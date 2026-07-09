savedcmd_iomem_rw.mod := printf '%s\n'   deadlocked.o | awk '!x[$$0]++ { print("./"$$0) }' > iomem_rw.mod
