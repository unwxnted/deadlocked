savedcmd_iomem_rw.o := ld -m elf_x86_64 -z noexecstack --no-warn-rwx-segments   -r -o iomem_rw.o @iomem_rw.mod  ; /usr/src/linux-7.1.3-1-obj/x86_64/default/tools/objtool/objtool --hacks=jump_label --hacks=noinstr --hacks=skylake --ibt --orc --retpoline --rethunk --sls --static-call --uaccess --prefix=16  --link  --module iomem_rw.o

iomem_rw.o: $(wildcard /usr/src/linux-7.1.3-1-obj/x86_64/default/tools/objtool/objtool)
