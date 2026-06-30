obj-m := iomem_rw.o
iomem_rw-objs := deadlocked.o

KDIR ?= /lib/modules/$(shell uname -r)/build

all:
	$(MAKE) -C $(KDIR) M=$(CURDIR) modules

clean:
	$(MAKE) -C $(KDIR) M=$(CURDIR) clean

install:
	sudo insmod iomem_rw.ko

uninstall:
	sudo rmmod iomem_rw

.PHONY: all clean install uninstall
