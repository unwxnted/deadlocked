obj-m += deadlocked.o

KDIR ?= /lib/modules/$(shell uname -r)/build

all:
	$(MAKE) -C $(KDIR) M=$(CURDIR) modules

clean:
	$(MAKE) -C $(KDIR) M=$(CURDIR) clean

install:
	sudo insmod deadlocked.ko

uninstall:
	sudo rmmod deadlocked

.PHONY: all clean install uninstall
