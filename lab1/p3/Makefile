mod_name=basemod

CC = gcc
CFLAGS = -g3 -std=c99 -Wall -pedantic
obj-m += basemod.o
obj-m += smartmod.o

all: p5

p5: multi

p4: test4
	make -C /lib/modules/$(shell uname -r)/build M=$(PWD) modules
	rmmod smartmod.ko 
	rmmod basemod.ko
	insmod basemod.ko
	insmod smartmod.ko

p3: test3
	make -C /lib/modules/$(shell uname -r)/build M=$(PWD) modules
	rmmod basemod.ko 
	insmod basemod.ko

multi: multi.o
	${CC} ${CFLAGS} -pthread -o $@ $^

test4: test4.o
	${CC} ${CFLAGS} -o $@ $^

test3: test3.o 
	${CC} ${CFLAGS} -o $@ $^

base:
	mknod /dev/mymem c 240 0 
	insmod basemod.ko

smart: base
	mknod /dev/mymem_smart c 241 0 
	insmod smartmod.ko

clean:
	make -C /lib/modules/$(shell uname -r)/build M=$(PWD) clean