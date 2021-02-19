CC = gcc
# CFLAGS = -g3 -std=c99 -Wall -pedantic


all: test7 

test7: test7.o
	gcc -Wall -pedantic -o $@ $^

test: test.o
	${CC} -g3 -std=c99 -Wall -pedantic -o $@ $^

cat:
	cat /usr/src/linux-4.17.4/capitalize_syscall/capitalize_syscall.c


cp:
	cp capitalize_syscall.c /usr/src/linux-4.17.4/capitalize_syscall/capitalize_syscall.c

header:
	vim /usr/src/linux-4.17.4/include/linux/syscalls.h

table:
	tail /usr/src/linux-4.17.4/arch/x86/entry/syscalls/syscall_64.tbl


# added in Makefile