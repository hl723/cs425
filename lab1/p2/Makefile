mod_name=hello2

obj-m += ${mod_name}.o

all:
	make -C /lib/modules/$(shell uname -r)/build M=$(PWD) modules
	rmmod ${mod_name}.ko 
	insmod ${mod_name}.ko

clean:
	make -C /lib/modules/$(shell uname -r)/build M=$(PWD) clean